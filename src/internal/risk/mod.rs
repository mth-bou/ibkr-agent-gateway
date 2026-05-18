//! Deterministic risk policy models for preview-only order workflows.

pub mod checks;
pub mod live_gate;
pub mod live_limits;
pub mod live_refusals;
pub mod policy;
pub mod policy_registry;
pub mod rate_counter;
pub mod validate;

pub use checks::run_risk_checks;
pub use live_gate::{LiveGate, LiveTradingGate};
pub use live_limits::{
    LiveFrequencyLimit, LiveLimitContext, LiveLimitPolicy, LiveSessionLimit, evaluate_live_limits,
    live_limit_context_for_order,
};
pub use live_refusals::{missing_gate_refusals, refusal_for_gate};
pub use policy::{RiskDecision, RiskPolicy, RiskRefusal, RiskWarning};
pub use policy_registry::{LivePolicyRegistry, StaticPolicyRegistry};
pub use rate_counter::apply_live_rate_counters;
pub use validate::validate_order_intent;
