//! Runtime configuration loading and validation for the gateway.

pub mod audit_retention;
pub mod live;
pub mod market_data;
pub mod order_preview;
pub mod paper;
pub mod remote_mcp;
pub mod sidecar;
pub mod validation;

use crate::internal::auth::{ScopeSet, is_local_scope};
use crate::internal::domain::{BrokerBackendKind, ErrorCode, GatewayError, MarketDataPolicy};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

pub use audit_retention::{AuditRetentionConfig, validate_audit_retention_config};
pub use live::{LiveTradingConfig, validate_live_trading_config};
pub use market_data::validate_market_data_policy;
pub use order_preview::{OrderPreviewConfig, validate_order_preview_config};
pub use paper::{PaperTradingConfig, validate_paper_trading_config};
pub use remote_mcp::{RemoteMcpConfig, validate_remote_mcp_config};
pub use sidecar::{SidecarConfig, validate_sidecar_config};
pub use validation::validate_tls_bypass_localhost_only;

/// Local gateway server mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServerMode {
    /// Local-only gateway.
    Local,
    /// Remote HTTP MCP gateway protected by OAuth/OIDC.
    RemoteMcp,
}

/// Audit account identifier mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountIdMode {
    /// HMAC account ids before audit storage.
    Hmac,
}

/// Audit storage kind.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditStorageConfig {
    /// `SQLite` connection string.
    Sqlite {
        /// Database URL or local file storage path.
        storage: String,
    },
}

/// Independent safety flags for feature classes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SafetyConfig {
    /// Whether write tools are enabled.
    pub write_tools_enabled: bool,
    /// Whether remote public MCP is enabled.
    pub remote_public_mcp_enabled: bool,
    /// Whether sidecar relay is enabled.
    pub sidecar_enabled: bool,
    /// Whether direct broker OAuth is enabled.
    pub direct_broker_oauth_enabled: bool,
    /// Whether live trading is enabled.
    pub live_trading_enabled: bool,
}

/// Local gateway configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GatewayConfiguration {
    /// Server mode.
    pub server_mode: ServerMode,
    /// Local bind address.
    pub bind_address: String,
    /// Broker backend.
    pub broker_backend: BrokerBackendKind,
    /// Client Portal Gateway base URL.
    #[schemars(with = "Option<String>")]
    pub client_portal_base_url: Option<Url>,
    /// Whether TLS is verified.
    pub verify_tls: bool,
    /// Keepalive interval in seconds.
    pub keepalive_interval_seconds: u64,
    /// Audit storage.
    pub audit_storage: AuditStorageConfig,
    /// Audit account id mode.
    pub audit_account_id_mode: AccountIdMode,
    /// Audit retention config.
    #[serde(default)]
    pub audit_retention: AuditRetentionConfig,
    /// Enabled local read scopes.
    pub enabled_read_scopes: ScopeSet,
    /// Market data policy.
    pub market_data_policy: MarketDataPolicy,
    /// Order preview configuration.
    #[serde(default)]
    pub order_preview: OrderPreviewConfig,
    /// Paper trading configuration.
    #[serde(default)]
    pub paper_trading: PaperTradingConfig,
    /// Live trading configuration.
    #[serde(default)]
    pub live_trading: LiveTradingConfig,
    /// Remote MCP configuration.
    #[serde(default)]
    pub remote_mcp: RemoteMcpConfig,
    /// Sidecar relay configuration.
    #[serde(default)]
    pub sidecar: SidecarConfig,
    /// Safety flags.
    pub safety: SafetyConfig,
}

impl GatewayConfiguration {
    /// Validates gateway configuration and fail-closed feature gates.
    pub fn validate(&self) -> Result<(), GatewayError> {
        if matches!(self.broker_backend, BrokerBackendKind::ClientPortalGateway)
            && self.client_portal_base_url.is_none()
        {
            return Err(GatewayError::new(
                ErrorCode::ConfigMissingBrokerBaseUrl,
                "broker.client_portal_gateway.base_url is required",
                false,
                Some("Configure the Client Portal Gateway base URL".to_string()),
            ));
        }

        if self.safety.write_tools_enabled {
            return Err(forbidden_config(
                ErrorCode::ConfigWriteToolsForbidden,
                "write_tools_enabled",
            ));
        }
        if self.safety.remote_public_mcp_enabled && !self.remote_mcp.enabled {
            return Err(forbidden_config(
                ErrorCode::ConfigRemoteMcpForbidden,
                "remote_public_mcp_enabled",
            ));
        }
        if self.safety.sidecar_enabled && !self.sidecar.enabled {
            return Err(forbidden_config(
                ErrorCode::ConfigSidecarForbidden,
                "sidecar_enabled",
            ));
        }
        if self.safety.direct_broker_oauth_enabled {
            return Err(forbidden_config(
                ErrorCode::ConfigRemoteMcpForbidden,
                "direct_broker_oauth_enabled",
            ));
        }
        if let Some(base_url) = &self.client_portal_base_url {
            validate_tls_bypass_localhost_only(base_url, self.verify_tls)?;
        }
        validate_market_data_policy(&self.market_data_policy)?;
        validate_audit_retention_config(
            &self.audit_retention,
            self.live_trading.enabled || self.safety.live_trading_enabled,
        )?;
        validate_order_preview_config(&self.order_preview)?;
        validate_paper_trading_config(&self.paper_trading)?;
        validate_live_trading_config(&self.live_trading, self.safety.live_trading_enabled)?;
        validate_remote_mcp_config(&self.remote_mcp, self.safety.remote_public_mcp_enabled)?;
        validate_sidecar_config(
            &self.sidecar,
            self.safety.sidecar_enabled,
            self.remote_mcp.enabled,
        )?;

        if let Some(scope) = self
            .remote_mcp
            .allowed_scopes
            .iter()
            .find(|scope| !is_local_scope(scope))
        {
            return Err(GatewayError::new(
                ErrorCode::AuthScopeNotAllowedInMvp,
                format!("Remote MCP scope is not known to the gateway: {scope}"),
                false,
                Some("Remove unknown remote scopes".to_string()),
            ));
        }

        Ok(())
    }
}

fn forbidden_config(code: ErrorCode, field: &str) -> GatewayError {
    GatewayError::new(
        code,
        format!("Configuration field is forbidden in the current mode: {field}"),
        false,
        Some(format!("Disable {field}")),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AccountIdMode, AuditRetentionConfig, AuditStorageConfig, GatewayConfiguration,
        LiveTradingConfig, OrderPreviewConfig, PaperTradingConfig, RemoteMcpConfig, SafetyConfig,
        ServerMode, SidecarConfig, validate_tls_bypass_localhost_only,
    };
    use crate::internal::auth::{HEALTH_READ, ScopeSet};
    use crate::internal::domain::{BrokerBackendKind, ErrorCode, MarketDataPolicy};
    use url::Url;

    #[test]
    fn permits_tls_bypass_for_localhost() {
        let parsed = Url::parse("https://localhost:5000/v1/api");
        let Ok(url) = parsed else {
            unreachable!("static localhost URL should parse");
        };

        assert!(validate_tls_bypass_localhost_only(&url, false).is_ok());
    }

    #[test]
    fn rejects_tls_bypass_for_remote_host() {
        let parsed = Url::parse("https://broker.example.com");
        let Ok(url) = parsed else {
            unreachable!("static remote URL should parse");
        };

        let error = validate_tls_bypass_localhost_only(&url, false);
        let Err(error) = error else {
            unreachable!("remote TLS bypass should fail");
        };
        assert_eq!(error.code, ErrorCode::ConfigTlsBypassNonLocalhost);
    }

    #[test]
    fn rejects_write_tools_enabled() {
        let scopes = ScopeSet::read_only([HEALTH_READ]);
        let Ok(scopes) = scopes else {
            unreachable!("read scope should be accepted");
        };
        let parsed = Url::parse("https://localhost:5000/v1/api");
        let Ok(base_url) = parsed else {
            unreachable!("static localhost URL should parse");
        };

        let config = GatewayConfiguration {
            server_mode: ServerMode::Local,
            bind_address: "127.0.0.1:8080".to_string(),
            broker_backend: BrokerBackendKind::ClientPortalGateway,
            client_portal_base_url: Some(base_url),
            verify_tls: false,
            keepalive_interval_seconds: 60,
            audit_storage: AuditStorageConfig::Sqlite {
                storage: "sqlite://ibkr-agent.db".to_string(),
            },
            audit_account_id_mode: AccountIdMode::Hmac,
            audit_retention: AuditRetentionConfig::default(),
            enabled_read_scopes: scopes,
            market_data_policy: MarketDataPolicy::default(),
            order_preview: OrderPreviewConfig::default(),
            paper_trading: PaperTradingConfig::default(),
            live_trading: LiveTradingConfig::default(),
            remote_mcp: RemoteMcpConfig::default(),
            sidecar: SidecarConfig::default(),
            safety: SafetyConfig {
                write_tools_enabled: true,
                ..SafetyConfig::default()
            },
        };

        let error = config.validate();
        let Err(error) = error else {
            unreachable!("write tools must be forbidden");
        };
        assert_eq!(error.code, ErrorCode::ConfigWriteToolsForbidden);
    }
}
