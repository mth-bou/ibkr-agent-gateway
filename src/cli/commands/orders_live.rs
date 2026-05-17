//! Live order commands.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use crate::internal::approval::ApprovalId;
use crate::internal::audit::{
    OrderIdempotencyOperation, OrderIdempotencyRecoveryContext, OrderIdempotencyWorkflow,
    SqliteAuditWriter,
};
use crate::internal::backend::IbkrBackend;
use crate::internal::config::LiveTradingConfig;
use crate::internal::domain::{
    AssetClass, BrokerOrderId, CurrencyCode, ErrorCode, GatewayError, LocalUserId, MarketSnapshot,
    Money, Quantity,
};
use crate::internal::orders::{
    IdempotencyKey, IdempotencyStore, KillSwitch, LiveCancelRequest, LiveSubmitRequest,
    LocalCandidateLiveWriter, PaperToLiveMigrationChecklist, cancel_live_order,
    stable_request_hash, submit_live_order,
};
use crate::internal::risk::{
    LiveFrequencyLimit, LiveLimitContext, LiveLimitPolicy, LiveSessionLimit, StaticPolicyRegistry,
};
use rust_decimal::Decimal;
use serde::Serialize;

const LIVE_SUBMIT_HUMAN_OUTPUT: &str = "live order candidate recorded";
const LIVE_CANCEL_HUMAN_OUTPUT: &str = "live cancel candidate recorded";

/// Runs a live submit command.
pub async fn submit(
    audit_writer: &SqliteAuditWriter,
    backend: &dyn IbkrBackend,
    account: &str,
    approval_id: &str,
    idempotency_key: &str,
    gates: LiveCommandGates,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let approval_id = ApprovalId::parse(approval_id)?;
    let approval = audit_writer
        .load_approval(&approval_id)
        .await?
        .ok_or_else(missing_approval)?;
    let preview_record = audit_writer
        .load_order_preview(&approval.preview_id)
        .await?
        .ok_or_else(missing_preview)?;
    let idempotency_key = IdempotencyKey::new(idempotency_key)?;
    let request_hash = stable_request_hash(
        "cli.live.submit",
        &LiveSubmitCliFingerprint {
            account,
            approval_id: approval_id.as_uuid().to_string(),
            gates,
        },
    )?;
    if let Some(payload) = audit_writer
        .replay_order_idempotency(&idempotency_key, &request_hash)
        .await?
    {
        return print_output(json, "live order candidate replayed", &payload);
    }

    let market_snapshot = backend
        .market_snapshot(&preview_record.validated_order.contract_id)
        .await?;
    let request = LiveSubmitRequest {
        order: preview_record.validated_order,
        approval,
        idempotency_key: idempotency_key.clone(),
        live_config: live_config(account_id, gates.enable_live, gates.acknowledge_migration),
        live_scope_granted: gates.live_scope,
        live_limit_context: live_limit_context(market_snapshot)?,
        kill_switch: kill_switch(gates.open_kill_switch),
        audit_available: true,
        migration_checklist: migration_checklist(gates.acknowledge_migration),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let policy_registry = StaticPolicyRegistry::single(live_limit_policy()?);
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Live,
        operation: OrderIdempotencyOperation::Submit,
        account_id: request.order.account_id.clone(),
        broker_order_id: None,
    };
    audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;
    let result =
        match submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await {
            Ok(result) => result,
            Err(error) => {
                handle_pending_order_error(audit_writer, &idempotency_key, &request_hash, &error)
                    .await?;
                return Err(error);
            }
        };
    let payload = serde_json::to_value(&result.lifecycle).map_err(|_| output_payload_error())?;
    audit_writer
        .insert_order_idempotency(&idempotency_key, &request_hash, &payload)
        .await?;
    audit_writer
        .upsert_live_order_pending(&result.lifecycle)
        .await?;
    audit_writer
        .mark_approval_consumed(&result.consumed_approval)
        .await?;
    print_output(json, LIVE_SUBMIT_HUMAN_OUTPUT, &result.lifecycle)
}

/// Runs a live cancel command.
pub async fn cancel(
    audit_writer: &SqliteAuditWriter,
    account: &str,
    broker_order_id: &str,
    idempotency_key: &str,
    gates: LiveCommandGates,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let Some(broker_order_id) = BrokerOrderId::new(broker_order_id) else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Broker order id is required",
            false,
            Some("Provide a broker order id".to_string()),
        ));
    };
    let idempotency_key = IdempotencyKey::new(idempotency_key)?;
    let request_hash = stable_request_hash(
        "cli.live.cancel",
        &LiveCancelCliFingerprint {
            account,
            broker_order_id: broker_order_id.as_str(),
            gates,
        },
    )?;
    if let Some(payload) = audit_writer
        .replay_order_idempotency(&idempotency_key, &request_hash)
        .await?
    {
        return print_output(json, "live cancel candidate replayed", &payload);
    }

    let request = LiveCancelRequest {
        account_id: account_id.clone(),
        broker_order_id,
        idempotency_key: idempotency_key.clone(),
        live_config: live_config(account_id, gates.enable_live, gates.acknowledge_migration),
        live_scope_granted: gates.live_scope,
        kill_switch: kill_switch(gates.open_kill_switch),
        audit_available: true,
        migration_checklist: migration_checklist(gates.acknowledge_migration),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Live,
        operation: OrderIdempotencyOperation::Cancel,
        account_id: request.account_id.clone(),
        broker_order_id: Some(request.broker_order_id.clone()),
    };
    audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;
    let result = match cancel_live_order(request, &writer, &mut idempotency_store).await {
        Ok(result) => result,
        Err(error) => {
            handle_pending_order_error(audit_writer, &idempotency_key, &request_hash, &error)
                .await?;
            return Err(error);
        }
    };
    let payload = serde_json::to_value(&result.lifecycle).map_err(|_| output_payload_error())?;
    audit_writer
        .insert_order_idempotency(&idempotency_key, &request_hash, &payload)
        .await?;
    audit_writer
        .remove_live_order_pending(
            &result.lifecycle.account_id,
            &result.lifecycle.broker_order_id,
        )
        .await?;
    print_output(json, LIVE_CANCEL_HUMAN_OUTPUT, &result.lifecycle)
}

/// Live CLI gate flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct LiveCommandGates {
    /// Explicit live enablement.
    pub enable_live: bool,
    /// Simulated live scope.
    pub live_scope: bool,
    /// Simulated kill switch state.
    pub open_kill_switch: bool,
    /// Paper-to-live checklist acknowledgement.
    pub acknowledge_migration: bool,
}

fn live_config(
    account_id: crate::internal::domain::AccountId,
    enabled: bool,
    migration_acknowledged: bool,
) -> LiveTradingConfig {
    LiveTradingConfig {
        enabled,
        allowed_accounts: vec![account_id],
        risk_policy_id: Some("cli-live-policy".to_string()),
        paper_to_live_checklist_acknowledged: migration_acknowledged,
        reconciler_interval_seconds: 5,
    }
}

fn kill_switch(open: bool) -> KillSwitch {
    if open {
        KillSwitch::open(LocalUserId::from_static("local-user"), "cli live smoke")
    } else {
        KillSwitch::closed(LocalUserId::from_static("local-user"), "closed by default")
    }
}

fn migration_checklist(acknowledged: bool) -> PaperToLiveMigrationChecklist {
    if acknowledged {
        PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static("local-user"))
    } else {
        PaperToLiveMigrationChecklist {
            paper_trading_validated: false,
            approvals_reviewed: false,
            limits_reviewed: false,
            kill_switch_tested: false,
            incident_runbook_reviewed: false,
            acknowledged_by: None,
            acknowledged_at: None,
        }
    }
}

fn live_limit_policy() -> Result<LiveLimitPolicy, GatewayError> {
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Static currency is invalid",
            false,
            None,
        ));
    };

    Ok(LiveLimitPolicy {
        policy_id: "cli-live-policy".to_string(),
        enabled: true,
        max_notional: Some(Money {
            amount: Decimal::new(1_000, 0),
            currency: currency.clone(),
        }),
        max_quantity: Some(Quantity::new(Decimal::new(10, 0))),
        allowed_symbols: vec!["AAPL".to_string()],
        allowed_asset_classes: vec![AssetClass::Stock],
        frequency_limit: Some(LiveFrequencyLimit {
            max_orders: 5,
            window_seconds: 300,
        }),
        session_limit: Some(LiveSessionLimit {
            max_orders_per_session: 20,
            max_session_notional: Some(Money {
                amount: Decimal::new(5_000, 0),
                currency,
            }),
        }),
        max_price_deviation_bps: Some(500),
        max_quote_age_seconds: Some(30),
    })
}

fn live_limit_context(market_snapshot: MarketSnapshot) -> Result<LiveLimitContext, GatewayError> {
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Static currency is invalid",
            false,
            None,
        ));
    };

    Ok(LiveLimitContext {
        symbol: "AAPL".to_string(),
        asset_class: AssetClass::Stock,
        submitted_in_window: 0,
        submitted_in_session: 0,
        session_notional: Some(Money {
            amount: Decimal::ZERO,
            currency,
        }),
        market_snapshot: Some(market_snapshot),
    })
}

#[cfg(test)]
mod tests {
    use super::{LIVE_CANCEL_HUMAN_OUTPUT, LIVE_SUBMIT_HUMAN_OUTPUT};

    #[test]
    fn live_human_outputs_describe_local_candidates_not_broker_execution() {
        assert_eq!(LIVE_SUBMIT_HUMAN_OUTPUT, "live order candidate recorded");
        assert_eq!(LIVE_CANCEL_HUMAN_OUTPUT, "live cancel candidate recorded");
        assert!(!LIVE_SUBMIT_HUMAN_OUTPUT.contains("submitted"));
        assert!(!LIVE_CANCEL_HUMAN_OUTPUT.contains("cancelled"));
    }
}

#[derive(Serialize)]
struct LiveSubmitCliFingerprint<'a> {
    account: &'a str,
    approval_id: String,
    gates: LiveCommandGates,
}

#[derive(Serialize)]
struct LiveCancelCliFingerprint<'a> {
    account: &'a str,
    broker_order_id: &'a str,
    gates: LiveCommandGates,
}

fn missing_approval() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Live submit requires an existing approval record",
        false,
        Some("Run approvals create and pass its approval_id".to_string()),
    )
}

fn missing_preview() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Live submit requires the approved preview to be present",
        false,
        Some("Create a fresh preview and approval before live submit".to_string()),
    )
}

async fn handle_pending_order_error(
    audit_writer: &SqliteAuditWriter,
    idempotency_key: &IdempotencyKey,
    request_hash: &str,
    error: &GatewayError,
) -> Result<(), GatewayError> {
    if is_writer_boundary_error(error.code) {
        audit_writer
            .mark_order_failed_after_writer(idempotency_key, request_hash, error)
            .await
    } else {
        audit_writer
            .delete_order_pending(idempotency_key, request_hash)
            .await
    }
}

const fn is_writer_boundary_error(code: ErrorCode) -> bool {
    matches!(
        code,
        ErrorCode::BrokerBackendUnavailable
            | ErrorCode::BrokerResponseInvalid
            | ErrorCode::BrokerSessionRequired
            | ErrorCode::OrderValidationFailed
    )
}

fn output_payload_error() -> GatewayError {
    GatewayError::new(
        ErrorCode::AuditWriteFailed,
        "Unable to serialize live lifecycle for idempotency",
        true,
        Some("Retry the live workflow".to_string()),
    )
}
