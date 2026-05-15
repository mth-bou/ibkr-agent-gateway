//! Live trading gate model.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Independent gates required before live submit/cancel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiveGate {
    /// Live feature config is enabled.
    FeatureEnabled,
    /// Target account is allowlisted for live trading.
    AccountAllowlisted,
    /// Caller has the required live write scope.
    LiveScopeGranted,
    /// The originating preview has not expired.
    PreviewUnexpired,
    /// Matching approval record exists.
    ApprovalRecord,
    /// Idempotency key is present.
    IdempotencyKey,
    /// Live risk policy passed.
    RiskPolicyPass,
    /// Kill switch is open.
    KillSwitchOpen,
    /// Audit storage is available.
    AuditAvailable,
    /// Paper-to-live checklist has been acknowledged.
    PaperToLiveChecklist,
}

/// Boolean state for every live trading gate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LiveTradingGate {
    /// Live feature config is enabled.
    pub feature_enabled: bool,
    /// Target account is allowlisted for live trading.
    pub account_allowlisted: bool,
    /// Caller has the required live write scope.
    pub live_scope_granted: bool,
    /// The originating preview has not expired.
    pub preview_unexpired: bool,
    /// Matching approval record exists.
    pub approval_record: bool,
    /// Idempotency key is present.
    pub idempotency_key: bool,
    /// Live risk policy passed.
    pub risk_policy_pass: bool,
    /// Kill switch is open.
    pub kill_switch_open: bool,
    /// Audit storage is available.
    pub audit_available: bool,
    /// Paper-to-live checklist has been acknowledged.
    pub paper_to_live_checklist: bool,
}

impl LiveTradingGate {
    /// Returns a gate set with every gate refused.
    #[must_use]
    pub const fn all_closed() -> Self {
        Self {
            feature_enabled: false,
            account_allowlisted: false,
            live_scope_granted: false,
            preview_unexpired: false,
            approval_record: false,
            idempotency_key: false,
            risk_policy_pass: false,
            kill_switch_open: false,
            audit_available: false,
            paper_to_live_checklist: false,
        }
    }

    /// Returns true when every live gate passes.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.feature_enabled
            && self.account_allowlisted
            && self.live_scope_granted
            && self.preview_unexpired
            && self.approval_record
            && self.idempotency_key
            && self.risk_policy_pass
            && self.kill_switch_open
            && self.audit_available
            && self.paper_to_live_checklist
    }

    /// Returns missing gates in deterministic order.
    #[must_use]
    pub fn missing_gates(&self) -> Vec<LiveGate> {
        let mut missing = Vec::new();
        push_if_missing(&mut missing, self.feature_enabled, LiveGate::FeatureEnabled);
        push_if_missing(
            &mut missing,
            self.account_allowlisted,
            LiveGate::AccountAllowlisted,
        );
        push_if_missing(
            &mut missing,
            self.live_scope_granted,
            LiveGate::LiveScopeGranted,
        );
        push_if_missing(
            &mut missing,
            self.preview_unexpired,
            LiveGate::PreviewUnexpired,
        );
        push_if_missing(&mut missing, self.approval_record, LiveGate::ApprovalRecord);
        push_if_missing(&mut missing, self.idempotency_key, LiveGate::IdempotencyKey);
        push_if_missing(
            &mut missing,
            self.risk_policy_pass,
            LiveGate::RiskPolicyPass,
        );
        push_if_missing(
            &mut missing,
            self.kill_switch_open,
            LiveGate::KillSwitchOpen,
        );
        push_if_missing(&mut missing, self.audit_available, LiveGate::AuditAvailable);
        push_if_missing(
            &mut missing,
            self.paper_to_live_checklist,
            LiveGate::PaperToLiveChecklist,
        );
        missing
    }
}

fn push_if_missing(missing: &mut Vec<LiveGate>, passed: bool, gate: LiveGate) {
    if !passed {
        missing.push(gate);
    }
}
