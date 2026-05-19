//! Live bracket submit flow.

use super::{
    IdempotencyKey, IdempotencyStore, KillSwitch, LiveOrderGroupWriter,
    PaperToLiveMigrationChecklist, idempotency::stable_request_hash,
    live_migration::validate_paper_to_live_migration,
};
use crate::internal::config::LiveTradingConfig;
use crate::internal::domain::{
    ErrorCode, GatewayError, OrderGroupLifecycle, OrderGroupStatus, ValidatedOrderGroup,
};

/// Live group submit request.
#[derive(Clone, Debug)]
pub struct LiveGroupSubmitRequest {
    /// Validated group.
    pub group: ValidatedOrderGroup,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Live trading config.
    pub live_config: LiveTradingConfig,
    /// Whether caller has live submit scope.
    pub live_scope_granted: bool,
    /// Kill switch.
    pub kill_switch: KillSwitch,
    /// Whether audit storage is available.
    pub audit_available: bool,
    /// Migration acknowledgement.
    pub migration_checklist: PaperToLiveMigrationChecklist,
}

/// Submits a live group through the configured writer.
pub async fn submit_live_group_order(
    request: LiveGroupSubmitRequest,
    writer: &dyn LiveOrderGroupWriter,
    idempotency_store: &mut IdempotencyStore,
) -> Result<OrderGroupLifecycle, GatewayError> {
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
        .contains(&request.group.account_id)
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
            "Live group submit scope is missing",
            "Grant the live submit scope",
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
            "Live group submit requires audit storage",
            "Restore audit storage before live trading",
        ));
    }
    validate_paper_to_live_migration(&request.migration_checklist)?;

    let request_hash = stable_request_hash("live.group.submit", &request.group)?;
    idempotency_store.record_or_replay(request.idempotency_key.clone(), request_hash)?;
    let receipt = writer
        .submit_live_group(&request.group, &request.idempotency_key)
        .await?;
    Ok(OrderGroupLifecycle {
        group_id: request.group.group_id,
        account_id: request.group.account_id,
        broker_order_ids: receipt.broker_order_ids,
        status: OrderGroupStatus::Submitted,
    })
}

fn live_error(code: ErrorCode, message: &str, user_action: &str) -> GatewayError {
    GatewayError::new(code, message, false, Some(user_action.to_string()))
}
