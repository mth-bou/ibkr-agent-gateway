//! Live modify flow guarded by independent gates.

use super::{
    IdempotencyKey, IdempotencyStore, KillSwitch, LiveOrderWriter, OrderModifyFields,
    PaperToLiveMigrationChecklist,
    idempotency::stable_request_hash,
    lifecycle::{LiveOrderLifecycleRecord, LiveOrderLifecycleStatus},
    live_migration::validate_paper_to_live_migration,
};
use crate::internal::approval::{ApprovalRecord, ApprovalStatus};
use crate::internal::config::LiveTradingConfig;
use crate::internal::domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError, ValidatedOrder};
use crate::internal::risk::{
    LiveLimitContext, LivePolicyRegistry, LiveTradingGate, RiskDecision, RiskRefusal,
    evaluate_live_limits, missing_gate_refusals,
};
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
    /// Approved replacement preview representing the post-modify order state.
    pub approved_order: ValidatedOrder,
    /// Matching approval record for the replacement preview.
    pub approval: ApprovalRecord,
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
    /// Live hard-limit context for the post-modify order state.
    pub live_limit_context: LiveLimitContext,
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
    /// Approval after successful one-time consumption.
    pub consumed_approval: ApprovalRecord,
}

/// Validates live modify gates and modifies the order via a [`LiveOrderWriter`].
pub async fn modify_live_order_without_local_idempotency(
    request: LiveModifyRequest,
    writer: &dyn LiveOrderWriter,
    policy_registry: &dyn LivePolicyRegistry,
) -> Result<LiveModifyResult, GatewayError> {
    modify_live_order_inner(request, writer, policy_registry, None).await
}

/// Validates live modify gates and local idempotency.
pub async fn modify_live_order(
    request: LiveModifyRequest,
    writer: &dyn LiveOrderWriter,
    policy_registry: &dyn LivePolicyRegistry,
    idempotency_store: &mut IdempotencyStore,
) -> Result<LiveModifyResult, GatewayError> {
    modify_live_order_inner(request, writer, policy_registry, Some(idempotency_store)).await
}

async fn modify_live_order_inner(
    request: LiveModifyRequest,
    writer: &dyn LiveOrderWriter,
    policy_registry: &dyn LivePolicyRegistry,
    idempotency_store: Option<&mut IdempotencyStore>,
) -> Result<LiveModifyResult, GatewayError> {
    let now = OffsetDateTime::now_utc();
    let Some(policy_id) = request.live_config.risk_policy_id.as_deref() else {
        return Err(live_error(
            ErrorCode::LiveGateMissing,
            "Live risk policy id is missing",
            "Configure live_trading.risk_policy_id",
        ));
    };
    let live_limit_policy = policy_registry.load_policy(policy_id).await?;
    request.changes.validate()?;
    if !request.changes.has_changes() {
        return Err(live_error(
            ErrorCode::OrderValidationFailed,
            "Live modify requires at least one bounded change",
            "Provide quantity, price, time-in-force, or trailing changes",
        ));
    }
    validate_modify_matches_approved_order(&request.changes, &request.approved_order)?;

    let limit_decision = evaluate_live_limits(
        &request.approved_order,
        &live_limit_policy,
        &request.live_limit_context,
    );
    let risk_policy_pass = matches!(&limit_decision, RiskDecision::Allow { .. });
    let approval_record_result = super::approval_gate::validate_approved_preview(
        &request.approval,
        &request.approved_order,
        now,
        ErrorCode::LiveGateMissing,
        "A matching approved preview is required for live modify",
        "Approve the replacement preview before live modify",
    );
    let approval_record = approval_record_result.is_ok();

    let migration_acknowledged =
        validate_paper_to_live_migration(&request.migration_checklist).is_ok();
    let account_allowlisted = request
        .live_config
        .allowed_accounts
        .contains(&request.account_id)
        && request.approved_order.account_id == request.account_id;
    let gate = LiveTradingGate {
        feature_enabled: request.live_config.enabled,
        account_allowlisted,
        live_scope_granted: request.live_scope_granted,
        preview_unexpired: request.approved_order.expires_at > now,
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

    let request_hash = stable_request_hash(
        "live.modify",
        &ModifyFingerprint {
            account_id: &request.account_id,
            broker_order_id: &request.broker_order_id,
            changes: &request.changes,
            approved_order: &request.approved_order,
            approval: &request.approval,
            live_limit_policy: &live_limit_policy,
            live_limit_context: &request.live_limit_context,
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
    let status = LiveOrderLifecycleStatus::from_modify_receipt_status(
        receipt.broker_status.as_deref(),
        receipt.accepted,
    );

    if !receipt.accepted && !status.is_terminal() {
        return Err(GatewayError::new(
            ErrorCode::BrokerResponseInvalid,
            "Broker did not accept live modify",
            false,
            Some("Inspect broker order status before retrying modify".to_string()),
        ));
    }
    let notional = if status.is_terminal() {
        None
    } else {
        live_order_notional(&request.approved_order)
    };
    let mut consumed_approval = request.approval.clone();
    consumed_approval.status = ApprovalStatus::Consumed;

    Ok(LiveModifyResult {
        lifecycle: LiveOrderLifecycleRecord {
            account_id: request.account_id,
            broker_order_id: receipt.broker_order_id,
            status,
            notional,
            execution_correlation: None,
            updated_at: now,
        },
        idempotency_key,
        consumed_approval,
    })
}

#[derive(Serialize)]
struct ModifyFingerprint<'a> {
    account_id: &'a AccountId,
    broker_order_id: &'a BrokerOrderId,
    changes: &'a OrderModifyFields,
    approved_order: &'a ValidatedOrder,
    approval: &'a ApprovalRecord,
    live_limit_policy: &'a crate::internal::risk::LiveLimitPolicy,
    live_limit_context: &'a LiveLimitContext,
}

fn live_error(code: ErrorCode, message: &str, user_action: &str) -> GatewayError {
    GatewayError::new(code, message, false, Some(user_action.to_string()))
}

fn live_order_notional(order: &ValidatedOrder) -> Option<crate::internal::domain::Money> {
    order
        .limit_price
        .as_ref()
        .map(|limit_price| crate::internal::domain::Money {
            amount: limit_price.amount * order.quantity.value,
            currency: limit_price.currency.clone(),
        })
}

fn live_limit_error(refusals: &[RiskRefusal]) -> GatewayError {
    GatewayError::new(
        ErrorCode::LiveLimitRefused,
        format!("Live limit refused modify: {}", refusals[0].code),
        false,
        refusals[0].user_action.clone(),
    )
}

fn gate_error(gate: &LiveTradingGate) -> GatewayError {
    let refusals = missing_gate_refusals(gate);
    let Some(first) = refusals.first() else {
        return GatewayError::new(
            ErrorCode::LiveGateMissing,
            "Live gate refused modify",
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
        format!("Live gate refused modify: {}", first.code),
        false,
        first.user_action.clone(),
    )
}

fn validate_modify_matches_approved_order(
    changes: &OrderModifyFields,
    approved_order: &ValidatedOrder,
) -> Result<(), GatewayError> {
    if let Some(quantity) = &changes.quantity
        && quantity != &approved_order.quantity
    {
        return Err(mismatched_approval("quantity"));
    }
    if let Some(limit_price) = &changes.limit_price
        && Some(limit_price) != approved_order.limit_price.as_ref()
    {
        return Err(mismatched_approval("limit_price"));
    }
    if let Some(stop_price) = &changes.stop_price
        && Some(stop_price) != approved_order.stop_price.as_ref()
    {
        return Err(mismatched_approval("stop_price"));
    }
    if let Some(time_in_force) = changes.time_in_force
        && time_in_force != approved_order.time_in_force
    {
        return Err(mismatched_approval("time_in_force"));
    }
    if let Some(trailing_amount) = &changes.trailing_amount
        && Some(trailing_amount) != approved_order.trailing_amount.as_ref()
    {
        return Err(mismatched_approval("trailing_amount"));
    }
    if let Some(trailing_percent) = changes.trailing_percent
        && Some(trailing_percent) != approved_order.trailing_percent
    {
        return Err(mismatched_approval("trailing_percent"));
    }
    Ok(())
}

fn mismatched_approval(field: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::ApprovalPreviewMismatch,
        format!("Live modify {field} does not match the approved replacement preview"),
        false,
        Some("Create an approval for the exact requested modify fields".to_string()),
    )
}
