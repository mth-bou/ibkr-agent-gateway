//! Sidecar relay configuration and fail-closed validation.

use crate::internal::domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

/// Sidecar relay configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SidecarConfig {
    /// Whether sidecar relay is enabled.
    pub enabled: bool,
    /// Remote relay endpoint.
    #[schemars(with = "Option<String>")]
    pub remote_relay_url: Option<Url>,
    /// Local Client Portal Gateway URL.
    #[schemars(with = "Option<String>")]
    pub local_client_portal_base_url: Option<Url>,
    /// Heartbeat interval in seconds.
    pub heartbeat_interval_seconds: u64,
    /// Heartbeat timeout in seconds.
    pub heartbeat_timeout_seconds: u64,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            remote_relay_url: None,
            local_client_portal_base_url: None,
            heartbeat_interval_seconds: 15,
            heartbeat_timeout_seconds: 45,
        }
    }
}

/// Validates sidecar relay configuration.
pub fn validate_sidecar_config(
    config: &SidecarConfig,
    safety_enabled: bool,
    remote_mcp_enabled: bool,
) -> Result<(), GatewayError> {
    if !config.enabled {
        if safety_enabled {
            return Err(GatewayError::new(
                ErrorCode::ConfigSidecarForbidden,
                "sidecar_enabled requires sidecar.enabled",
                false,
                Some("Enable sidecar.enabled or disable the safety flag".to_string()),
            ));
        }
        return Ok(());
    }

    if !safety_enabled {
        return Err(GatewayError::new(
            ErrorCode::ConfigSidecarForbidden,
            "sidecar.enabled requires sidecar_enabled",
            false,
            Some("Set the explicit sidecar_enabled safety flag".to_string()),
        ));
    }
    if !remote_mcp_enabled {
        return Err(GatewayError::new(
            ErrorCode::ConfigSidecarForbidden,
            "sidecar relay requires remote MCP to be enabled",
            false,
            Some("Enable remote MCP OAuth before sidecar relay".to_string()),
        ));
    }
    if config.remote_relay_url.is_none() {
        return Err(missing("sidecar.remote_relay_url"));
    }
    if config.local_client_portal_base_url.is_none() {
        return Err(missing("sidecar.local_client_portal_base_url"));
    }
    if config.heartbeat_interval_seconds == 0
        || config.heartbeat_timeout_seconds <= config.heartbeat_interval_seconds
    {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            "sidecar heartbeat timeout must be greater than interval",
            false,
            Some("Increase sidecar.heartbeat_timeout_seconds".to_string()),
        ));
    }

    Ok(())
}

fn missing(field: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::ConfigInvalid,
        format!("Sidecar configuration is missing {field}"),
        false,
        Some(format!("Configure {field} before enabling sidecar relay")),
    )
}
