//! Live submit flow guarded by independent gates.

use crate::{
    IdempotencyKey, KillSwitch, PaperToLiveMigrationChecklist,
    lifecycle::{LiveOrderLifecycleRecord, LiveOrderLifecycleStatus},
    live_migration::validate_paper_to_live_migration,
};
use ibkr_approval::{ApprovalRecord, ApprovalStatus};
use ibkr_config::LiveTradingConfig;
use ibkr_domain::{BrokerOrderId, ErrorCode, GatewayError, ValidatedOrder};
use ibkr_risk::{
    LiveLimitContext, LiveLimitPolicy, LiveTradingGate, RiskDecision, evaluate_live_limits,
    missing_gate_refusals,
};
use time::OffsetDateTime;

/// Live submit request.
pub struct LiveSubmitRequest {
    /// Validated order candidate.
    pub order: ValidatedOrder,
    /// Matching approval record.
    pub approval: ApprovalRecord,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Live trading config.
    pub live_config: LiveTradingConfig,
    /// Whether caller has the live submit scope.
    pub live_scope_granted: bool,
    /// Live hard-limit policy.
    pub live_limit_policy: LiveLimitPolicy,
    /// Live hard-limit context.
    pub live_limit_context: LiveLimitContext,
    /// Current kill switch state.
    pub kill_switch: KillSwitch,
    /// Whether audit storage is available.
    pub audit_available: bool,
    /// Paper-to-live migration checklist.
    pub migration_checklist: PaperToLiveMigrationChecklist,
}

/// Live submit result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveSubmitResult {
    /// Lifecycle record.
    pub lifecycle: LiveOrderLifecycleRecord,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
}

/// Validates all live gates and records a live submit candidate.
pub fn submit_live_order(request: LiveSubmitRequest) -> Result<LiveSubmitResult, GatewayError> {
    let now = OffsetDateTime::now_utc();
    let limit_decision = evaluate_live_limits(
        &request.order,
        &request.live_limit_policy,
        &request.live_limit_context,
    );
    let risk_policy_pass = request.live_limit_policy.enabled
        && request
            .live_config
            .risk_policy_id
            .as_deref()
            .is_some_and(|policy_id| policy_id == request.live_limit_policy.policy_id);

    let approval_record = request.approval.status == ApprovalStatus::Approved
        && request.approval.account_id == request.order.account_id
        && request.approval.expires_at > now;
    let migration_acknowledged =
        validate_paper_to_live_migration(&request.migration_checklist).is_ok();
    let gate = LiveTradingGate {
        feature_enabled: request.live_config.enabled,
        account_allowlisted: request
            .live_config
            .allowed_accounts
            .contains(&request.order.account_id),
        live_scope_granted: request.live_scope_granted,
        preview_unexpired: request.order.expires_at > now,
        approval_record,
        idempotency_key: true,
        risk_policy_pass,
        kill_switch_open: request.kill_switch.is_open(),
        audit_available: request.audit_available,
        paper_to_live_checklist: migration_acknowledged,
    };

    if !gate.is_open() {
        return Err(gate_error(&gate));
    }

    if let RiskDecision::Refuse { refusals } = limit_decision {
        return Err(GatewayError::new(
            ErrorCode::LiveLimitRefused,
            format!("Live limit refused order: {}", refusals[0].code),
            false,
            refusals[0].user_action.clone(),
        ));
    }

    Ok(LiveSubmitResult {
        lifecycle: LiveOrderLifecycleRecord {
            account_id: request.order.account_id,
            broker_order_id: BrokerOrderId::from_static("live-order-local"),
            status: LiveOrderLifecycleStatus::Submitted,
            execution_correlation: None,
            updated_at: now,
        },
        idempotency_key: request.idempotency_key,
    })
}

fn gate_error(gate: &LiveTradingGate) -> GatewayError {
    let refusals = missing_gate_refusals(gate);
    let Some(first) = refusals.first() else {
        return GatewayError::new(
            ErrorCode::LiveGateMissing,
            "Live gate refused order",
            false,
            Some("Review live trading gates".to_string()),
        );
    };

    let code = match first.code.as_str() {
        "LIVE_FEATURE_DISABLED" => ErrorCode::LiveTradingDisabled,
        "LIVE_KILL_SWITCH_CLOSED" => ErrorCode::LiveKillSwitchClosed,
        "LIVE_MIGRATION_CHECKLIST_MISSING" => ErrorCode::LiveMigrationRequired,
        _ => ErrorCode::LiveGateMissing,
    };

    GatewayError::new(
        code,
        format!("Live gate refused order: {}", first.code),
        false,
        first.user_action.clone(),
    )
}
