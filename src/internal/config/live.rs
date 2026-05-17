//! Live trading configuration.

use crate::internal::domain::{AccountId, ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Live trading config.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LiveTradingConfig {
    /// Whether live trading is enabled in the feature config.
    pub enabled: bool,
    /// Explicit live account allowlist.
    pub allowed_accounts: Vec<AccountId>,
    /// Stable risk policy id expected for live writes.
    pub risk_policy_id: Option<String>,
    /// Whether the paper-to-live checklist has been acknowledged.
    pub paper_to_live_checklist_acknowledged: bool,
    /// Poll interval for live order lifecycle reconciliation.
    #[serde(default = "default_live_reconciler_interval_seconds")]
    pub reconciler_interval_seconds: u64,
}

impl Default for LiveTradingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_accounts: Vec::new(),
            risk_policy_id: None,
            paper_to_live_checklist_acknowledged: false,
            reconciler_interval_seconds: default_live_reconciler_interval_seconds(),
        }
    }
}

/// Validates live trading config and the independent safety flag.
pub fn validate_live_trading_config(
    config: &LiveTradingConfig,
    safety_live_enabled: bool,
) -> Result<(), GatewayError> {
    if safety_live_enabled && !config.enabled {
        return Err(live_config_error(
            "Live trading safety flag is enabled but live trading config is disabled",
            "Enable live_trading.enabled or disable safety.live_trading_enabled",
        ));
    }

    if config.enabled && !safety_live_enabled {
        return Err(live_config_error(
            "Live trading requires the independent safety flag",
            "Set safety.live_trading_enabled only after completing live readiness checks",
        ));
    }

    if config.enabled && config.allowed_accounts.is_empty() {
        return Err(live_config_error(
            "Live trading requires an explicit account allowlist",
            "Add at least one live account id to the allowlist",
        ));
    }

    if config.enabled && config.risk_policy_id.is_none() {
        return Err(live_config_error(
            "Live trading requires an explicit risk policy id",
            "Configure live_trading.risk_policy_id",
        ));
    }

    if config.enabled && !config.paper_to_live_checklist_acknowledged {
        return Err(live_config_error(
            "Live trading requires paper-to-live checklist acknowledgement",
            "Complete and acknowledge the paper-to-live checklist",
        ));
    }

    Ok(())
}

const fn default_live_reconciler_interval_seconds() -> u64 {
    5
}

fn live_config_error(message: &str, user_action: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::ConfigLiveTradingForbidden,
        message,
        false,
        Some(user_action.to_string()),
    )
}
