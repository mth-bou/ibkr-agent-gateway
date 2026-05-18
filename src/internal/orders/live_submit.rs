//! Live submit flow guarded by independent gates.

use super::{
    IdempotencyKey, IdempotencyStore, KillSwitch, LiveOrderWriter, PaperToLiveMigrationChecklist,
    idempotency::stable_request_hash,
    lifecycle::{LiveOrderLifecycleRecord, LiveOrderLifecycleStatus},
    live_migration::validate_paper_to_live_migration,
};
use crate::internal::approval::{ApprovalRecord, ApprovalStatus};
use crate::internal::config::LiveTradingConfig;
use crate::internal::domain::{ErrorCode, GatewayError, ValidatedOrder};
use crate::internal::risk::{
    LiveLimitContext, LivePolicyRegistry, LiveTradingGate, RiskDecision, RiskRefusal,
    evaluate_live_limits, missing_gate_refusals,
};
use serde::Serialize;
use time::OffsetDateTime;

/// Live submit request.
#[derive(Clone, Debug)]
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
    /// Approval after successful one-time consumption.
    pub consumed_approval: ApprovalRecord,
}

/// Validates all live gates and submits the order through a [`LiveOrderWriter`].
///
/// All deterministic refusals (config, scopes, allowlists, kill switch, risk
/// policy, approval, migration checklist) are evaluated before the writer is
/// invoked, so a misconfigured deployment never reaches the broker.
pub async fn submit_live_order(
    request: LiveSubmitRequest,
    writer: &dyn LiveOrderWriter,
    policy_registry: &dyn LivePolicyRegistry,
    idempotency_store: &mut IdempotencyStore,
) -> Result<LiveSubmitResult, GatewayError> {
    let now = OffsetDateTime::now_utc();
    let Some(policy_id) = request.live_config.risk_policy_id.as_deref() else {
        return Err(GatewayError::new(
            ErrorCode::LiveGateMissing,
            "Live risk policy id is missing",
            false,
            Some("Configure live_trading.risk_policy_id".to_string()),
        ));
    };
    let live_limit_policy = policy_registry.load_policy(policy_id).await?;
    let limit_decision = evaluate_live_limits(
        &request.order,
        &live_limit_policy,
        &request.live_limit_context,
    );
    let risk_policy_pass = matches!(&limit_decision, RiskDecision::Allow { .. });

    let approval_record_result = super::approval_gate::validate_approved_preview(
        &request.approval,
        &request.order,
        now,
        ErrorCode::LiveGateMissing,
        "A matching approved preview is required for live submit",
        "Approve the preview before live submit",
    );
    let approval_record = approval_record_result.is_ok();
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
        let gate_without_risk_policy = LiveTradingGate {
            risk_policy_pass: true,
            ..gate
        };
        if !risk_policy_pass
            && gate_without_risk_policy.is_open()
            && let RiskDecision::Refuse { refusals } = limit_decision
        {
            return Err(live_limit_error(&refusals));
        }

        let gate_without_approval = LiveTradingGate {
            approval_record: true,
            ..gate
        };
        if !approval_record
            && gate_without_approval.is_open()
            && let Err(error) = approval_record_result
        {
            return Err(error);
        }
        return Err(gate_error(&gate));
    }
    approval_record_result?;

    if let RiskDecision::Refuse { refusals } = limit_decision {
        return Err(live_limit_error(&refusals));
    }

    let request_hash = stable_request_hash(
        "live.submit",
        &LiveSubmitFingerprint {
            order: &request.order,
            approval: &request.approval,
            live_limit_policy: &live_limit_policy,
            live_limit_context: &request.live_limit_context,
        },
    )?;
    let idempotency_key = request.idempotency_key.clone();
    idempotency_store.record_or_replay(idempotency_key.clone(), request_hash)?;

    let receipt = writer.submit_live(&request.order, &idempotency_key).await?;

    let mut consumed_approval = request.approval.clone();
    consumed_approval.status = ApprovalStatus::Consumed;

    Ok(LiveSubmitResult {
        lifecycle: LiveOrderLifecycleRecord {
            account_id: request.order.account_id,
            broker_order_id: receipt.broker_order_id,
            status: LiveOrderLifecycleStatus::Submitted,
            execution_correlation: None,
            updated_at: now,
        },
        idempotency_key,
        consumed_approval,
    })
}

fn live_limit_error(refusals: &[RiskRefusal]) -> GatewayError {
    GatewayError::new(
        ErrorCode::LiveLimitRefused,
        format!("Live limit refused order: {}", refusals[0].code),
        false,
        refusals[0].user_action.clone(),
    )
}

#[derive(Serialize)]
struct LiveSubmitFingerprint<'a> {
    order: &'a ValidatedOrder,
    approval: &'a ApprovalRecord,
    live_limit_policy: &'a crate::internal::risk::LiveLimitPolicy,
    live_limit_context: &'a LiveLimitContext,
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
