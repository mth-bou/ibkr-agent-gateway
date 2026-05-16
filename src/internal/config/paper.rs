//! Paper trading configuration.

use crate::internal::domain::{AccountId, ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Paper trading config.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PaperTradingConfig {
    /// Whether paper trading is enabled.
    pub enabled: bool,
    /// Explicit paper account allowlist.
    pub allowed_accounts: Vec<AccountId>,
}

/// Validates paper trading config.
pub fn validate_paper_trading_config(config: &PaperTradingConfig) -> Result<(), GatewayError> {
    if config.enabled && config.allowed_accounts.is_empty() {
        return Err(GatewayError::new(
            ErrorCode::PaperTradingDisabled,
            "Paper trading requires an explicit account allowlist",
            false,
            Some("Add at least one paper account id to the allowlist".to_string()),
        ));
    }

    Ok(())
}
