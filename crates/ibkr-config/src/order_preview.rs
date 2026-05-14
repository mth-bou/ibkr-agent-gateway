//! Order preview configuration.

use ibkr_domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Preview-only feature configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OrderPreviewConfig {
    /// Whether order preview is enabled.
    pub enabled: bool,
    /// Preview expiration in seconds.
    pub preview_expiration_seconds: u64,
}

impl Default for OrderPreviewConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preview_expiration_seconds: 300,
        }
    }
}

/// Validates order preview configuration.
pub fn validate_order_preview_config(config: &OrderPreviewConfig) -> Result<(), GatewayError> {
    if config.preview_expiration_seconds == 0 {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            "order_preview.preview_expiration_seconds must be positive",
            false,
            Some("Set a positive preview expiration".to_string()),
        ));
    }

    Ok(())
}
