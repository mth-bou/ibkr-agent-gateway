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

/// Default number of remote MCP requests accepted per rate-limit window.
pub const DEFAULT_RATE_LIMIT_MAX_REQUESTS: u32 = 120;

/// Default remote MCP rate-limit window, in seconds.
pub const DEFAULT_RATE_LIMIT_WINDOW_SECONDS: u64 = 60;

/// Default number of concurrent remote MCP HTTP connections.
pub const DEFAULT_MAX_CONNECTIONS: usize = 64;

/// Maximum accepted remote MCP rate-limit window, in seconds.
pub const MAX_RATE_LIMIT_WINDOW_SECONDS: u64 = 3_600;

/// Maximum accepted number of requests per remote MCP rate-limit window.
pub const MAX_RATE_LIMIT_MAX_REQUESTS: u32 = 100_000;

/// Maximum accepted number of concurrent remote MCP HTTP connections.
pub const MAX_MAX_CONNECTIONS: usize = 4_096;

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
    /// Number of authorization attempts allowed per rate-limit window.
    #[serde(default = "default_rate_limit_max_requests")]
    pub rate_limit_max_requests: u32,
    /// Rate-limit window duration in seconds.
    #[serde(default = "default_rate_limit_window_seconds")]
    pub rate_limit_window_seconds: u64,
    /// Maximum concurrent HTTP connections handled by the remote MCP listener.
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
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
            rate_limit_max_requests: DEFAULT_RATE_LIMIT_MAX_REQUESTS,
            rate_limit_window_seconds: DEFAULT_RATE_LIMIT_WINDOW_SECONDS,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            token_id_hmac_secret: None,
        }
    }
}

const fn default_rate_limit_max_requests() -> u32 {
    DEFAULT_RATE_LIMIT_MAX_REQUESTS
}

const fn default_rate_limit_window_seconds() -> u64 {
    DEFAULT_RATE_LIMIT_WINDOW_SECONDS
}

const fn default_max_connections() -> usize {
    DEFAULT_MAX_CONNECTIONS
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
    if config.rate_limit_max_requests == 0
        || config.rate_limit_max_requests > MAX_RATE_LIMIT_MAX_REQUESTS
    {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "remote_mcp.rate_limit_max_requests must be between 1 and {MAX_RATE_LIMIT_MAX_REQUESTS}"
            ),
            false,
            Some("Configure a bounded positive remote MCP rate limit".to_string()),
        ));
    }
    if config.rate_limit_window_seconds == 0
        || config.rate_limit_window_seconds > MAX_RATE_LIMIT_WINDOW_SECONDS
    {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            format!(
                "remote_mcp.rate_limit_window_seconds must be between 1 and {MAX_RATE_LIMIT_WINDOW_SECONDS}"
            ),
            false,
            Some("Configure a bounded positive remote MCP rate-limit window".to_string()),
        ));
    }
    if config.max_connections == 0 || config.max_connections > MAX_MAX_CONNECTIONS {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            format!("remote_mcp.max_connections must be between 1 and {MAX_MAX_CONNECTIONS}"),
            false,
            Some("Configure a bounded positive remote MCP connection limit".to_string()),
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
    use super::{
        DEFAULT_MAX_CONNECTIONS, DEFAULT_RATE_LIMIT_MAX_REQUESTS,
        DEFAULT_RATE_LIMIT_WINDOW_SECONDS, MAX_CLOCK_SKEW_SECONDS, MAX_MAX_CONNECTIONS,
        MAX_RATE_LIMIT_MAX_REQUESTS, MAX_RATE_LIMIT_WINDOW_SECONDS, RemoteMcpConfig,
        validate_remote_mcp_config,
    };
    use crate::internal::domain::ErrorCode;
    use serde_json::json;

    #[test]
    fn default_clock_skew_is_within_bound() {
        let config = RemoteMcpConfig::default();
        assert!(config.clock_skew_seconds <= MAX_CLOCK_SKEW_SECONDS);
    }

    #[test]
    fn default_remote_mcp_limits_are_bounded() {
        let config = RemoteMcpConfig::default();
        assert_eq!(
            config.rate_limit_max_requests,
            DEFAULT_RATE_LIMIT_MAX_REQUESTS
        );
        assert_eq!(
            config.rate_limit_window_seconds,
            DEFAULT_RATE_LIMIT_WINDOW_SECONDS
        );
        assert_eq!(config.max_connections, DEFAULT_MAX_CONNECTIONS);
    }

    #[test]
    fn deserializes_missing_new_limit_fields_with_defaults()
    -> Result<(), Box<dyn std::error::Error>> {
        let config: RemoteMcpConfig = serde_json::from_value(json!({
            "enabled": false,
            "bind_address": "127.0.0.1:8080",
            "resource": null,
            "issuer": null,
            "jwks_url": null,
            "metadata_url": null,
            "audiences": [],
            "allowed_scopes": [],
            "clock_skew_seconds": 60
        }))?;

        assert_eq!(
            config.rate_limit_max_requests,
            DEFAULT_RATE_LIMIT_MAX_REQUESTS
        );
        assert_eq!(
            config.rate_limit_window_seconds,
            DEFAULT_RATE_LIMIT_WINDOW_SECONDS
        );
        assert_eq!(config.max_connections, DEFAULT_MAX_CONNECTIONS);
        Ok(())
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

    #[test]
    fn rejects_invalid_rate_limit_max_requests() {
        for rate_limit_max_requests in [0, MAX_RATE_LIMIT_MAX_REQUESTS + 1] {
            let config = RemoteMcpConfig {
                rate_limit_max_requests,
                ..RemoteMcpConfig::default()
            };
            let Err(error) = validate_remote_mcp_config(&config, false) else {
                unreachable!("invalid rate limit max requests must be rejected");
            };
            assert_eq!(error.code, ErrorCode::ConfigInvalid);
        }
    }

    #[test]
    fn rejects_invalid_rate_limit_window_seconds() {
        for rate_limit_window_seconds in [0, MAX_RATE_LIMIT_WINDOW_SECONDS + 1] {
            let config = RemoteMcpConfig {
                rate_limit_window_seconds,
                ..RemoteMcpConfig::default()
            };
            let Err(error) = validate_remote_mcp_config(&config, false) else {
                unreachable!("invalid rate limit window must be rejected");
            };
            assert_eq!(error.code, ErrorCode::ConfigInvalid);
        }
    }

    #[test]
    fn rejects_invalid_max_connections() {
        for max_connections in [0, MAX_MAX_CONNECTIONS + 1] {
            let config = RemoteMcpConfig {
                max_connections,
                ..RemoteMcpConfig::default()
            };
            let Err(error) = validate_remote_mcp_config(&config, false) else {
                unreachable!("invalid max connections must be rejected");
            };
            assert_eq!(error.code, ErrorCode::ConfigInvalid);
        }
    }
}
