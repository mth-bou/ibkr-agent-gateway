//! Live trading refusal matrix.

use crate::{
    live_gate::{LiveGate, LiveTradingGate},
    policy::RiskRefusal,
};

/// Converts missing live gates to deterministic refusals.
#[must_use]
pub fn missing_gate_refusals(gate: &LiveTradingGate) -> Vec<RiskRefusal> {
    gate.missing_gates()
        .into_iter()
        .map(refusal_for_gate)
        .collect()
}

/// Converts one missing live gate to a deterministic refusal.
#[must_use]
pub fn refusal_for_gate(gate: LiveGate) -> RiskRefusal {
    let (code, message, user_action) = match gate {
        LiveGate::FeatureEnabled => (
            "LIVE_FEATURE_DISABLED",
            "Live trading is disabled",
            "Enable live trading explicitly in configuration",
        ),
        LiveGate::AccountAllowlisted => (
            "LIVE_ACCOUNT_NOT_ALLOWLISTED",
            "Account is not in the live trading allowlist",
            "Use an explicitly allowlisted live account",
        ),
        LiveGate::LiveScopeGranted => (
            "LIVE_SCOPE_MISSING",
            "Live write scope is missing",
            "Grant the required live trading scope",
        ),
        LiveGate::PreviewUnexpired => (
            "LIVE_PREVIEW_EXPIRED",
            "Live submit requires an unexpired preview",
            "Create and approve a fresh order preview",
        ),
        LiveGate::ApprovalRecord => (
            "LIVE_APPROVAL_MISSING",
            "Live submit requires a matching approval record",
            "Approve the preview before live submit",
        ),
        LiveGate::IdempotencyKey => (
            "LIVE_IDEMPOTENCY_KEY_MISSING",
            "Live write requires an idempotency key",
            "Provide an idempotency key",
        ),
        LiveGate::RiskPolicyPass => (
            "LIVE_RISK_POLICY_REFUSED",
            "Live risk policy did not pass",
            "Review the live risk refusal",
        ),
        LiveGate::KillSwitchOpen => (
            "LIVE_KILL_SWITCH_CLOSED",
            "Live kill switch is closed",
            "Open the live kill switch only after operator review",
        ),
        LiveGate::AuditAvailable => (
            "LIVE_AUDIT_UNAVAILABLE",
            "Live writes require audit storage",
            "Restore audit storage before live trading",
        ),
        LiveGate::PaperToLiveChecklist => (
            "LIVE_MIGRATION_CHECKLIST_MISSING",
            "Paper-to-live checklist has not been acknowledged",
            "Complete the paper-to-live checklist",
        ),
    };

    RiskRefusal {
        code: code.to_string(),
        message: message.to_string(),
        user_action: Some(user_action.to_string()),
    }
}
