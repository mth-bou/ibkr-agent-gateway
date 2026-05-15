//! Remote MCP configuration and fail-closed validation.

use ibkr_domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

/// Remote MCP OAuth/OIDC configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RemoteMcpConfig {
    /// Whether remote HTTP MCP is enabled.
    pub enabled: bool,
    /// HTTP bind address.
    pub bind_address: String,
    /// Public protected-resource identifier.
    #[schemars(with = "Option<String>")]
    pub resource: Option<Url>,
    /// Expected OIDC issuer.
    #[schemars(with = "Option<String>")]
    pub issuer: Option<Url>,
    /// JWKS endpoint URL.
    #[schemars(with = "Option<String>")]
    pub jwks_url: Option<Url>,
    /// Optional authorization server metadata URL.
    #[schemars(with = "Option<String>")]
    pub metadata_url: Option<Url>,
    /// Accepted audiences/resources.
    pub audiences: Vec<String>,
    /// Gateway scopes that remote tokens may grant.
    pub allowed_scopes: Vec<String>,
    /// Accepted clock skew for time claims.
    pub clock_skew_seconds: u64,
}

impl Default for RemoteMcpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_address: "127.0.0.1:8080".to_string(),
            resource: None,
            issuer: None,
            jwks_url: None,
            metadata_url: None,
            audiences: Vec::new(),
            allowed_scopes: Vec::new(),
            clock_skew_seconds: 60,
        }
    }
}

/// Validates remote MCP configuration.
pub fn validate_remote_mcp_config(
    config: &RemoteMcpConfig,
    safety_enabled: bool,
) -> Result<(), GatewayError> {
    if !config.enabled {
        if safety_enabled {
            return Err(GatewayError::new(
                ErrorCode::ConfigRemoteMcpForbidden,
                "remote_public_mcp_enabled requires remote_mcp.enabled",
                false,
                Some("Enable remote_mcp.enabled or disable the safety flag".to_string()),
            ));
        }
        return Ok(());
    }

    if !safety_enabled {
        return Err(GatewayError::new(
            ErrorCode::ConfigRemoteMcpForbidden,
            "remote_mcp.enabled requires remote_public_mcp_enabled",
            false,
            Some("Set the explicit remote_public_mcp_enabled safety flag".to_string()),
        ));
    }
    if config.resource.is_none() {
        return Err(missing("remote_mcp.resource"));
    }
    if config.issuer.is_none() {
        return Err(missing("remote_mcp.issuer"));
    }
    if config.jwks_url.is_none() {
        return Err(missing("remote_mcp.jwks_url"));
    }
    if config.audiences.is_empty() {
        return Err(missing("remote_mcp.audiences"));
    }
    if config.allowed_scopes.is_empty() {
        return Err(missing("remote_mcp.allowed_scopes"));
    }

    Ok(())
}

fn missing(field: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::ConfigInvalid,
        format!("Remote MCP configuration is missing {field}"),
        false,
        Some(format!("Configure {field} before enabling remote MCP")),
    )
}
