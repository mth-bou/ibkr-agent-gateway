//! Live modify flow guarded by independent gates.

use super::{
    IdempotencyKey, IdempotencyStore, KillSwitch, LiveOrderWriter, OrderModifyFields,
    PaperToLiveMigrationChecklist,
    idempotency::stable_request_hash,
    lifecycle::{LiveOrderLifecycleRecord, LiveOrderLifecycleStatus},
    live_migration::validate_paper_to_live_migration,
};
use crate::internal::config::LiveTradingConfig;
use crate::internal::domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError};
use serde::Serialize;
use time::OffsetDateTime;

/// Live modify request.
#[derive(Clone, Debug)]
pub struct LiveModifyRequest {
    /// Account id.
    pub account_id: AccountId,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Bounded changes.
    pub changes: OrderModifyFields,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Live trading config.
    pub live_config: LiveTradingConfig,
    /// Whether caller has the live modify scope.
    pub live_scope_granted: bool,
    /// Current kill switch state.
    pub kill_switch: KillSwitch,
    /// Whether audit storage is available.
    pub audit_available: bool,
    /// Paper-to-live migration checklist.
    pub migration_checklist: PaperToLiveMigrationChecklist,
}

/// Live modify result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveModifyResult {
    /// Lifecycle record.
    pub lifecycle: LiveOrderLifecycleRecord,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
}

/// Validates live modify gates and modifies the order via a [`LiveOrderWriter`].
pub async fn modify_live_order_without_local_idempotency(
    request: LiveModifyRequest,
    writer: &dyn LiveOrderWriter,
) -> Result<LiveModifyResult, GatewayError> {
    modify_live_order_inner(request, writer, None).await
}

/// Validates live modify gates and local idempotency.
pub async fn modify_live_order(
    request: LiveModifyRequest,
    writer: &dyn LiveOrderWriter,
    idempotency_store: &mut IdempotencyStore,
) -> Result<LiveModifyResult, GatewayError> {
    modify_live_order_inner(request, writer, Some(idempotency_store)).await
}

async fn modify_live_order_inner(
    request: LiveModifyRequest,
    writer: &dyn LiveOrderWriter,
    idempotency_store: Option<&mut IdempotencyStore>,
) -> Result<LiveModifyResult, GatewayError> {
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
            "Live modify scope is missing",
            "Grant the live modify scope",
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
            "Live modify requires audit storage",
            "Restore audit storage before live trading",
        ));
    }
    if !request.changes.has_changes() {
        return Err(live_error(
            ErrorCode::OrderValidationFailed,
            "Live modify requires at least one bounded change",
            "Provide quantity, price, time-in-force, or trailing changes",
        ));
    }

    validate_paper_to_live_migration(&request.migration_checklist)?;

    let request_hash = stable_request_hash(
        "live.modify",
        &ModifyFingerprint {
            account_id: &request.account_id,
            broker_order_id: &request.broker_order_id,
            changes: &request.changes,
        },
    )?;
    let idempotency_key = request.idempotency_key.clone();
    if let Some(idempotency_store) = idempotency_store {
        idempotency_store.record_or_replay(idempotency_key.clone(), request_hash)?;
    }
    let receipt = writer
        .modify_live(
            &request.account_id,
            &request.broker_order_id,
            &request.changes,
            &idempotency_key,
        )
        .await?;

    Ok(LiveModifyResult {
        lifecycle: LiveOrderLifecycleRecord {
            account_id: request.account_id,
            broker_order_id: receipt.broker_order_id,
            status: LiveOrderLifecycleStatus::Open,
            notional: None,
            execution_correlation: None,
            updated_at: OffsetDateTime::now_utc(),
        },
        idempotency_key,
    })
}

#[derive(Serialize)]
struct ModifyFingerprint<'a> {
    account_id: &'a AccountId,
    broker_order_id: &'a BrokerOrderId,
    changes: &'a OrderModifyFields,
}

fn live_error(code: ErrorCode, message: &str, user_action: &str) -> GatewayError {
    GatewayError::new(code, message, false, Some(user_action.to_string()))
}
