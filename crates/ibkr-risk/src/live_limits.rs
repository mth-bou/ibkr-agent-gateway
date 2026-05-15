//! Live trading hard-limit policies.

use crate::policy::{RiskDecision, RiskRefusal};
use ibkr_domain::{AssetClass, Money, Quantity, ValidatedOrder};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Frequency limit for live submissions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LiveFrequencyLimit {
    /// Maximum order count.
    pub max_orders: u32,
    /// Window size in seconds.
    pub window_seconds: u64,
}

/// Session limit for live submissions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LiveSessionLimit {
    /// Maximum order count in the current session.
    pub max_orders_per_session: u32,
    /// Maximum session notional.
    pub max_session_notional: Option<Money>,
}

/// Deterministic hard limits for live trading.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LiveLimitPolicy {
    /// Stable policy id.
    pub policy_id: String,
    /// Whether the live policy is enabled.
    pub enabled: bool,
    /// Maximum notional for a single order.
    pub max_notional: Option<Money>,
    /// Maximum quantity for a single order.
    pub max_quantity: Option<Quantity>,
    /// Allowed symbols.
    pub allowed_symbols: Vec<String>,
    /// Allowed asset classes.
    pub allowed_asset_classes: Vec<AssetClass>,
    /// Frequency limit.
    pub frequency_limit: Option<LiveFrequencyLimit>,
    /// Session limit.
    pub session_limit: Option<LiveSessionLimit>,
}

impl Default for LiveLimitPolicy {
    fn default() -> Self {
        Self {
            policy_id: "default-live-disabled".to_string(),
            enabled: false,
            max_notional: None,
            max_quantity: None,
            allowed_symbols: Vec::new(),
            allowed_asset_classes: Vec::new(),
            frequency_limit: None,
            session_limit: None,
        }
    }
}

/// Context needed to evaluate live limits that are not present on the order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LiveLimitContext {
    /// Resolved trading symbol.
    pub symbol: String,
    /// Resolved asset class.
    pub asset_class: AssetClass,
    /// Number of orders already submitted in the active frequency window.
    pub submitted_in_window: u32,
    /// Number of orders already submitted in the active session.
    pub submitted_in_session: u32,
    /// Session notional before the candidate order.
    pub session_notional: Option<Money>,
}

/// Evaluates live hard limits for an already validated order.
#[must_use]
pub fn evaluate_live_limits(
    order: &ValidatedOrder,
    policy: &LiveLimitPolicy,
    context: &LiveLimitContext,
) -> RiskDecision {
    let mut refusals = Vec::new();

    if !policy.enabled {
        refusals.push(refusal(
            "LIVE_LIMIT_POLICY_DISABLED",
            "Live risk policy is disabled",
            "Enable an explicit live risk policy",
        ));
    }

    if policy.allowed_symbols.is_empty() || !policy.allowed_symbols.contains(&context.symbol) {
        refusals.push(refusal(
            "LIVE_SYMBOL_REFUSED",
            "Symbol is not allowed by live policy",
            "Use a symbol from the live allowlist",
        ));
    }

    if policy.allowed_asset_classes.is_empty()
        || !policy.allowed_asset_classes.contains(&context.asset_class)
    {
        refusals.push(refusal(
            "LIVE_ASSET_CLASS_REFUSED",
            "Asset class is not allowed by live policy",
            "Use an allowed live asset class",
        ));
    }

    if let Some(max_quantity) = &policy.max_quantity
        && order.quantity.value > max_quantity.value
    {
        refusals.push(refusal(
            "LIVE_QUANTITY_LIMIT_REFUSED",
            "Quantity exceeds live policy maximum",
            "Reduce the requested quantity",
        ));
    }

    if let (Some(limit_price), Some(max_notional)) = (&order.limit_price, &policy.max_notional) {
        let notional = limit_price.amount * order.quantity.value;
        if notional > max_notional.amount {
            refusals.push(refusal(
                "LIVE_NOTIONAL_LIMIT_REFUSED",
                "Estimated notional exceeds live policy maximum",
                "Reduce quantity or price",
            ));
        }
    }

    if let Some(limit) = policy.frequency_limit
        && context.submitted_in_window >= limit.max_orders
    {
        refusals.push(refusal(
            "LIVE_FREQUENCY_LIMIT_REFUSED",
            "Live order frequency limit has been reached",
            "Wait for the live frequency window to reset",
        ));
    }

    if let Some(limit) = &policy.session_limit {
        if context.submitted_in_session >= limit.max_orders_per_session {
            refusals.push(refusal(
                "LIVE_SESSION_ORDER_LIMIT_REFUSED",
                "Live session order limit has been reached",
                "Stop trading or start a new approved session",
            ));
        }

        if let (Some(session_limit), Some(session_notional), Some(limit_price)) = (
            &limit.max_session_notional,
            &context.session_notional,
            &order.limit_price,
        ) {
            let candidate_notional = limit_price.amount * order.quantity.value;
            if session_notional.amount + candidate_notional > session_limit.amount {
                refusals.push(refusal(
                    "LIVE_SESSION_NOTIONAL_LIMIT_REFUSED",
                    "Live session notional limit would be exceeded",
                    "Reduce order size or stop trading for the session",
                ));
            }
        }
    }

    if refusals.is_empty() {
        RiskDecision::Allow {
            warnings: Vec::new(),
        }
    } else {
        RiskDecision::Refuse { refusals }
    }
}

fn refusal(code: &str, message: &str, user_action: &str) -> RiskRefusal {
    RiskRefusal {
        code: code.to_string(),
        message: message.to_string(),
        user_action: Some(user_action.to_string()),
    }
}
