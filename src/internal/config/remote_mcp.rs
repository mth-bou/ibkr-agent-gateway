//! Remote MCP configuration and fail-closed validation.

use crate::internal::domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

/// Maximum accepted clock-skew tolerance, in seconds.
///
/// Caps how far an operator may relax JWT `exp`/`nbf` enforcement. A value
/// above this bound risks silently disabling expiry checks once the skew is
/// added to `claims.exp` (see review finding H-1, 2026-05-17).
pub const MAX_CLOCK_SKEW_SECONDS: u64 = 300;

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
    /// Secret injected from config/env/secret manager to hash remote token ids for audit.
    #[schemars(skip)]
    #[serde(default, skip_serializing)]
    pub token_id_hmac_secret: Option<String>,
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
            token_id_hmac_secret: None,
        }
    }
}

/// Validates remote MCP configuration.
pub fn validate_remote_mcp_config(
    config: &RemoteMcpConfig,
    safety_enabled: bool,
) -> Result<(), GatewayError> {
    if config.clock_skew_seconds > MAX_CLOCK_SKEW_SECONDS {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            format!("remote_mcp.clock_skew_seconds must be at most {MAX_CLOCK_SKEW_SECONDS}"),
            false,
            Some(format!(
                "Lower clock_skew_seconds to {MAX_CLOCK_SKEW_SECONDS} or below"
            )),
        ));
    }

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
    if config
        .token_id_hmac_secret
        .as_deref()
        .is_none_or(|secret| secret.trim().is_empty())
    {
        return Err(missing("remote_mcp.token_id_hmac_secret"));
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

#[cfg(test)]
mod tests {
    use super::{MAX_CLOCK_SKEW_SECONDS, RemoteMcpConfig, validate_remote_mcp_config};
    use crate::internal::domain::ErrorCode;

    #[test]
    fn default_clock_skew_is_within_bound() {
        let config = RemoteMcpConfig::default();
        assert!(config.clock_skew_seconds <= MAX_CLOCK_SKEW_SECONDS);
    }

    #[test]
    fn rejects_clock_skew_exceeding_cap() {
        let config = RemoteMcpConfig {
            clock_skew_seconds: MAX_CLOCK_SKEW_SECONDS + 1,
            ..RemoteMcpConfig::default()
        };
        let Err(error) = validate_remote_mcp_config(&config, false) else {
            unreachable!("clock skew above cap must be rejected");
        };
        assert_eq!(error.code, ErrorCode::ConfigInvalid);
    }

    #[test]
    fn rejects_u64_max_clock_skew() {
        let config = RemoteMcpConfig {
            clock_skew_seconds: u64::MAX,
            ..RemoteMcpConfig::default()
        };
        let Err(error) = validate_remote_mcp_config(&config, false) else {
            unreachable!("u64::MAX clock skew must be rejected");
        };
        assert_eq!(error.code, ErrorCode::ConfigInvalid);
    }

    #[test]
    fn accepts_clock_skew_at_cap() {
        let config = RemoteMcpConfig {
            clock_skew_seconds: MAX_CLOCK_SKEW_SECONDS,
            ..RemoteMcpConfig::default()
        };
        let outcome = validate_remote_mcp_config(&config, false);
        assert!(outcome.is_ok());
    }
}
