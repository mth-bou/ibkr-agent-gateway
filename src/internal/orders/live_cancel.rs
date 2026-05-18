//! Live cancel flow guarded by independent gates.

use super::{
    IdempotencyKey, IdempotencyStore, KillSwitch, LiveOrderWriter, PaperToLiveMigrationChecklist,
    idempotency::stable_request_hash,
    lifecycle::{LiveOrderLifecycleRecord, LiveOrderLifecycleStatus},
    live_migration::validate_paper_to_live_migration,
};
use crate::internal::config::LiveTradingConfig;
use crate::internal::domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError};
use serde::Serialize;
use time::OffsetDateTime;

/// Live cancel request.
#[derive(Clone, Debug)]
pub struct LiveCancelRequest {
    /// Account id.
    pub account_id: AccountId,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Live trading config.
    pub live_config: LiveTradingConfig,
    /// Whether caller has the live cancel scope.
    pub live_scope_granted: bool,
    /// Current kill switch state.
    pub kill_switch: KillSwitch,
    /// Whether audit storage is available.
    pub audit_available: bool,
    /// Paper-to-live migration checklist.
    pub migration_checklist: PaperToLiveMigrationChecklist,
}

/// Live cancel result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveCancelResult {
    /// Lifecycle record.
    pub lifecycle: LiveOrderLifecycleRecord,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
}

/// Validates live cancel gates and cancels the order via a [`LiveOrderWriter`].
pub async fn cancel_live_order(
    request: LiveCancelRequest,
    writer: &dyn LiveOrderWriter,
    idempotency_store: &mut IdempotencyStore,
) -> Result<LiveCancelResult, GatewayError> {
    cancel_live_order_inner(request, writer, Some(idempotency_store)).await
}

/// Validates and cancels a live order when durable caller-level idempotency is already enforced.
pub(crate) async fn cancel_live_order_without_local_idempotency(
    request: LiveCancelRequest,
    writer: &dyn LiveOrderWriter,
) -> Result<LiveCancelResult, GatewayError> {
    cancel_live_order_inner(request, writer, None).await
}

async fn cancel_live_order_inner(
    request: LiveCancelRequest,
    writer: &dyn LiveOrderWriter,
    idempotency_store: Option<&mut IdempotencyStore>,
) -> Result<LiveCancelResult, GatewayError> {
    if !request.live_config.enabled {
        return Err(live_error(
            ErrorCode::LiveTradingDisabled,
            "Live trading is disabled",
            "Enable live trading explicitly in configuration",
        ));
    }

    if !request
        .live_config
        .allowed_accounts
        .contains(&request.account_id)
    {
        return Err(live_error(
            ErrorCode::LiveGateMissing,
            "Account is not in the live trading allowlist",
            "Use an explicitly allowlisted live account",
        ));
    }

    if !request.live_scope_granted {
        return Err(live_error(
            ErrorCode::LiveGateMissing,
            "Live cancel scope is missing",
            "Grant the live cancel scope",
        ));
    }

    if !request.kill_switch.is_open() {
        return Err(live_error(
            ErrorCode::LiveKillSwitchClosed,
            "Live kill switch is closed",
            "Open the live kill switch only after operator review",
        ));
    }

    if !request.audit_available {
        return Err(live_error(
            ErrorCode::LiveGateMissing,
            "Live cancel requires audit storage",
            "Restore audit storage before live trading",
        ));
    }

    validate_paper_to_live_migration(&request.migration_checklist)?;

    let request_hash = stable_request_hash(
        "live.cancel",
        &LiveCancelFingerprint {
            account_id: &request.account_id,
            broker_order_id: &request.broker_order_id,
        },
    )?;
    let idempotency_key = request.idempotency_key.clone();
    if let Some(idempotency_store) = idempotency_store {
        idempotency_store.record_or_replay(idempotency_key.clone(), request_hash)?;
    }

    let receipt = writer
        .cancel_live(
            &request.account_id,
            &request.broker_order_id,
            &idempotency_key,
        )
        .await?;
    let status = LiveOrderLifecycleStatus::from_cancel_receipt_status(
        receipt.broker_status.as_deref(),
        receipt.accepted,
    );

    if !receipt.accepted && !status.is_terminal() {
        return Err(GatewayError::new(
            ErrorCode::BrokerResponseInvalid,
            "Broker did not accept live cancel",
            false,
            Some("Inspect broker order status before retrying cancel".to_string()),
        ));
    }

    Ok(LiveCancelResult {
        lifecycle: LiveOrderLifecycleRecord {
            account_id: request.account_id,
            broker_order_id: receipt.broker_order_id,
            status,
            execution_correlation: None,
            updated_at: OffsetDateTime::now_utc(),
        },
        idempotency_key,
    })
}

#[derive(Serialize)]
struct LiveCancelFingerprint<'a> {
    account_id: &'a AccountId,
    broker_order_id: &'a BrokerOrderId,
}

fn live_error(code: ErrorCode, message: &str, user_action: &str) -> GatewayError {
    GatewayError::new(code, message, false, Some(user_action.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{LiveCancelRequest, cancel_live_order};
    use crate::internal::config::LiveTradingConfig;
    use crate::internal::domain::{
        AccountId, BrokerOrderId, ErrorCode, GatewayError, ValidatedOrder,
    };
    use crate::internal::orders::{
        IdempotencyKey, IdempotencyStore, KillSwitch, LiveCancelReceipt, LiveOrderLifecycleStatus,
        LiveOrderWriter, LiveSubmitReceipt, PaperToLiveMigrationChecklist,
    };
    use async_trait::async_trait;

    #[tokio::test]
    async fn live_cancel_preserves_pending_cancel_status() -> Result<(), Box<dyn std::error::Error>>
    {
        let account = AccountId::from_static("DU1234567");
        let broker_order = BrokerOrderId::from_static("broker-1");
        let request = live_cancel_request(account.clone(), broker_order.clone())?;
        let writer = StaticCancelWriter {
            accepted: true,
            broker_status: Some("PendingCancel".to_string()),
        };
        let mut idempotency_store = IdempotencyStore::default();

        let result = cancel_live_order(request, &writer, &mut idempotency_store).await?;

        assert_eq!(result.lifecycle.account_id, account);
        assert_eq!(result.lifecycle.broker_order_id, broker_order);
        assert_eq!(
            result.lifecycle.status,
            LiveOrderLifecycleStatus::PendingCancel
        );
        Ok(())
    }

    #[tokio::test]
    async fn live_cancel_rejects_unaccepted_active_status() -> Result<(), Box<dyn std::error::Error>>
    {
        let request = live_cancel_request(
            AccountId::from_static("DU1234567"),
            BrokerOrderId::from_static("broker-1"),
        )?;
        let writer = StaticCancelWriter {
            accepted: false,
            broker_status: Some("Submitted".to_string()),
        };
        let mut idempotency_store = IdempotencyStore::default();

        let error = cancel_live_order(request, &writer, &mut idempotency_store)
            .await
            .err()
            .ok_or("unaccepted active status should fail")?;

        assert_eq!(error.code, ErrorCode::BrokerResponseInvalid);
        Ok(())
    }

    fn live_cancel_request(
        account_id: AccountId,
        broker_order_id: BrokerOrderId,
    ) -> Result<LiveCancelRequest, GatewayError> {
        Ok(LiveCancelRequest {
            account_id: account_id.clone(),
            broker_order_id,
            idempotency_key: IdempotencyKey::new("live-cancel-test")?,
            live_config: LiveTradingConfig {
                enabled: true,
                allowed_accounts: vec![account_id],
                risk_policy_id: Some("live-policy".to_string()),
                paper_to_live_checklist_acknowledged: true,
                reconciler_interval_seconds: 5,
            },
            live_scope_granted: true,
            kill_switch: KillSwitch::open(
                crate::internal::domain::LocalUserId::from_static("operator"),
                "test",
            ),
            audit_available: true,
            migration_checklist: PaperToLiveMigrationChecklist::acknowledged(
                crate::internal::domain::LocalUserId::from_static("operator"),
            ),
        })
    }

    struct StaticCancelWriter {
        accepted: bool,
        broker_status: Option<String>,
    }

    #[async_trait]
    impl LiveOrderWriter for StaticCancelWriter {
        async fn submit_live(
            &self,
            _order: &ValidatedOrder,
            _idempotency_key: &IdempotencyKey,
        ) -> Result<LiveSubmitReceipt, GatewayError> {
            Err(GatewayError::new(
                ErrorCode::BrokerResponseInvalid,
                "submit is not used in this test",
                false,
                None,
            ))
        }

        async fn cancel_live(
            &self,
            _account_id: &AccountId,
            broker_order_id: &BrokerOrderId,
            _idempotency_key: &IdempotencyKey,
        ) -> Result<LiveCancelReceipt, GatewayError> {
            Ok(LiveCancelReceipt {
                broker_order_id: broker_order_id.clone(),
                accepted: self.accepted,
                broker_status: self.broker_status.clone(),
            })
        }
    }
}
