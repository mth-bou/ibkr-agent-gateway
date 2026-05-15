//! Live cancel flow guarded by independent gates.

use crate::{
    IdempotencyKey, KillSwitch, PaperToLiveMigrationChecklist,
    lifecycle::{LiveOrderLifecycleRecord, LiveOrderLifecycleStatus},
    live_migration::validate_paper_to_live_migration,
};
use ibkr_config::LiveTradingConfig;
use ibkr_domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError};
use time::OffsetDateTime;

/// Live cancel request.
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

/// Validates live cancel gates and records a cancel candidate.
pub fn cancel_live_order(request: LiveCancelRequest) -> Result<LiveCancelResult, GatewayError> {
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

    Ok(LiveCancelResult {
        lifecycle: LiveOrderLifecycleRecord {
            account_id: request.account_id,
            broker_order_id: request.broker_order_id,
            status: LiveOrderLifecycleStatus::Cancelled,
            execution_correlation: None,
            updated_at: OffsetDateTime::now_utc(),
        },
        idempotency_key: request.idempotency_key,
    })
}

fn live_error(code: ErrorCode, message: &str, user_action: &str) -> GatewayError {
    GatewayError::new(code, message, false, Some(user_action.to_string()))
}
