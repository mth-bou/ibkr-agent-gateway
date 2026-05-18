//! CLI runtime construction from defaults or a config file.

use crate::internal::audit::{AuditHmacKey, SqliteAuditWriter};
use crate::internal::auth::{LOCAL_SCOPES, ScopeSet};
use crate::internal::backend::{BackendFactoryConfig, IbkrBackend, create_backend};
use crate::internal::config::{
    AuditRetentionConfig, LiveTradingConfig, RemoteMcpConfig, SidecarConfig,
    validate_audit_retention_config, validate_live_trading_config, validate_remote_mcp_config,
    validate_sidecar_config, validate_tls_bypass_localhost_only,
};
use crate::internal::cpapi::{ClientPortalClient, ClientPortalLiveWriter};
use crate::internal::domain::{AccountId, BrokerBackendKind, ErrorCode, GatewayError};
use crate::internal::orders::{
    LiveOrderWriter, LocalCandidateLiveWriter, RefusingLiveWriter,
    recover_pending_order_idempotency,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use url::Url;

const DEFAULT_DEV_AUDIT_KEY: &[u8] = b"ibkr-agent-gateway-local-dev-audit-key-0001";
static NEXT_TEST_AUDIT_ID: AtomicU64 = AtomicU64::new(1);

/// Runtime dependencies shared by all CLI commands.
pub struct CliRuntime {
    /// Broker backend selected for this invocation.
    pub backend: Box<dyn IbkrBackend>,
    /// Durable audit and workflow state.
    pub audit_writer: SqliteAuditWriter,
    /// Audit HMAC key shared with backend account hashing.
    pub audit_hmac_key: Arc<AuditHmacKey>,
    /// Local scopes granted to this CLI invocation.
    pub scopes: ScopeSet,
    /// Live lifecycle reconciliation interval in seconds.
    pub live_reconciler_interval_seconds: u64,
    /// Validated live trading configuration.
    pub live_trading_config: LiveTradingConfig,
    /// Remote MCP configuration loaded from the CLI config file.
    pub remote_mcp_config: RemoteMcpConfig,
    /// Sidecar relay configuration loaded from the CLI config file.
    pub sidecar_config: SidecarConfig,
    backend_kind: BrokerBackendKind,
    client_portal_base_url: Option<Url>,
    verify_tls: bool,
}

impl CliRuntime {
    /// Builds the runtime from an optional CLI config path.
    pub async fn load(config_path: Option<&Path>) -> Result<Self, GatewayError> {
        let config = match config_path {
            Some(path) => CliRuntimeConfig::from_file(path)?,
            None => CliRuntimeConfig::dev_default()?,
        };

        let audit_hmac_key = Arc::new(AuditHmacKey::new(config.audit_hmac_secret)?);
        let backend = create_backend(BackendFactoryConfig {
            backend: config.backend,
            fixture_root: config.fixture_root.clone(),
            client_portal_base_url: config.client_portal_base_url.clone(),
            verify_tls: config.verify_tls,
            audit_hmac_key: audit_hmac_key.clone(),
        })?;
        let audit_writer =
            SqliteAuditWriter::connect(&config.audit_database_url, audit_hmac_key.clone()).await?;
        recover_pending_order_idempotency(&audit_writer, backend.as_ref()).await?;
        audit_writer
            .rebuild_live_order_pending_from_idempotency()
            .await?;

        Ok(Self {
            backend,
            audit_writer,
            audit_hmac_key,
            scopes: config.scopes,
            live_reconciler_interval_seconds: config.live_reconciler_interval_seconds,
            live_trading_config: config.live_trading_config,
            remote_mcp_config: config.remote_mcp_config,
            sidecar_config: config.sidecar_config,
            backend_kind: config.backend,
            client_portal_base_url: config.client_portal_base_url,
            verify_tls: config.verify_tls,
        })
    }

    /// Builds the live order writer selected by the CLI.
    pub fn live_order_writer(
        &self,
        choice: crate::cli::LiveBrokerChoice,
    ) -> Result<Box<dyn LiveOrderWriter>, GatewayError> {
        match choice {
            crate::cli::LiveBrokerChoice::LocalCandidate => Ok(Box::new(LocalCandidateLiveWriter)),
            crate::cli::LiveBrokerChoice::Refusing => Ok(Box::new(RefusingLiveWriter)),
            crate::cli::LiveBrokerChoice::ClientPortal => {
                if self.backend_kind != BrokerBackendKind::ClientPortalGateway {
                    return Err(GatewayError::new(
                        ErrorCode::ConfigInvalid,
                        "Client Portal live writer requires the Client Portal Gateway backend",
                        false,
                        Some("Use broker.backend: client_portal_gateway or --live-broker local-candidate".to_string()),
                    ));
                }
                let Some(base_url) = self.client_portal_base_url.clone() else {
                    return Err(GatewayError::new(
                        ErrorCode::ConfigMissingBrokerBaseUrl,
                        "Client Portal Gateway base URL is required for live writer",
                        false,
                        Some("Configure broker.base_url".to_string()),
                    ));
                };
                Ok(Box::new(ClientPortalLiveWriter::new(
                    ClientPortalClient::new(base_url, self.verify_tls)?,
                )))
            }
        }
    }
}

struct CliRuntimeConfig {
    backend: BrokerBackendKind,
    fixture_root: PathBuf,
    client_portal_base_url: Option<Url>,
    verify_tls: bool,
    audit_database_url: String,
    audit_hmac_secret: Vec<u8>,
    scopes: ScopeSet,
    live_reconciler_interval_seconds: u64,
    live_trading_config: LiveTradingConfig,
    remote_mcp_config: RemoteMcpConfig,
    sidecar_config: SidecarConfig,
}

impl CliRuntimeConfig {
    fn dev_default() -> Result<Self, GatewayError> {
        let audit_path = default_dev_audit_path();
        Ok(Self {
            backend: BrokerBackendKind::Fake,
            fixture_root: PathBuf::from("tests/fixtures/cpapi"),
            client_portal_base_url: None,
            verify_tls: true,
            audit_database_url: sqlite_url_from_path(&audit_path)?,
            audit_hmac_secret: DEFAULT_DEV_AUDIT_KEY.to_vec(),
            scopes: ScopeSet::local_with_live(LOCAL_SCOPES.iter().copied())?,
            live_reconciler_interval_seconds: default_live_reconciler_interval_seconds(),
            live_trading_config: LiveTradingConfig::default(),
            remote_mcp_config: RemoteMcpConfig::default(),
            sidecar_config: SidecarConfig::default(),
        })
    }

    fn from_file(path: &Path) -> Result<Self, GatewayError> {
        if !path.is_file() {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                format!("Config file does not exist: {}", path.display()),
                false,
                Some("Pass a valid --config path".to_string()),
            ));
        }

        let file_config = config::Config::builder()
            .add_source(config::File::from(path))
            .build()
            .and_then(config::Config::try_deserialize::<CliConfigFile>)
            .map_err(|error| {
                GatewayError::new(
                    ErrorCode::ConfigInvalid,
                    format!("Unable to load CLI config: {error}"),
                    false,
                    Some("Check the YAML config shape and values".to_string()),
                )
            })?;

        let backend = parse_backend(&file_config.broker.backend)?;
        let client_portal_base_url = match backend {
            BrokerBackendKind::Fake => None,
            BrokerBackendKind::ClientPortalGateway => Some(parse_url(
                file_config.broker.base_url.as_deref(),
                "broker.base_url",
            )?),
        };
        let verify_tls = !file_config
            .broker
            .allow_insecure_tls_for_localhost
            .unwrap_or(false);
        if let Some(base_url) = &client_portal_base_url {
            validate_tls_bypass_localhost_only(base_url, verify_tls)?;
        }
        let audit_database_url = sqlite_url_from_config_path(&file_config.audit.sqlite_path)?;
        let audit_hmac_secret = secret_from_env(&file_config.audit.hmac_secret_env)?;
        let scopes = ScopeSet::local_with_live(file_config.auth.enabled_scopes)?;
        let safety_live_enabled = file_config.safety.live_trading_enabled;
        let remote_mcp_safety_enabled = file_config.safety.remote_mcp_enabled;
        let sidecar_safety_enabled = file_config.safety.sidecar_enabled;
        let live_trading_config = file_config.live_trading.into_live_config()?;
        validate_live_trading_config(&live_trading_config, safety_live_enabled)?;
        let audit_retention = file_config.audit.audit_retention_config();
        validate_audit_retention_config(
            &audit_retention,
            live_trading_config.enabled || safety_live_enabled,
        )?;
        let live_reconciler_interval_seconds =
            live_trading_config.reconciler_interval_seconds.max(1);
        let remote_mcp_config = file_config.remote_mcp.into_remote_mcp_config()?;
        validate_remote_mcp_config(&remote_mcp_config, remote_mcp_safety_enabled)?;
        let sidecar_config = file_config.sidecar.into_sidecar_config()?;
        validate_sidecar_config(
            &sidecar_config,
            sidecar_safety_enabled,
            remote_mcp_config.enabled,
        )?;

        Ok(Self {
            backend,
            fixture_root: file_config
                .broker
                .fixture_root
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("tests/fixtures/cpapi")),
            client_portal_base_url,
            verify_tls,
            audit_database_url,
            audit_hmac_secret,
            scopes,
            live_reconciler_interval_seconds,
            live_trading_config,
            remote_mcp_config,
            sidecar_config,
        })
    }
}

const fn default_live_reconciler_interval_seconds() -> u64 {
    5
}

fn default_dev_audit_path() -> PathBuf {
    let filename = if running_under_cargo_test() {
        let id = NEXT_TEST_AUDIT_ID.fetch_add(1, Ordering::Relaxed);
        format!("ibkr-agent-gateway-cli-{}-{id}.sqlite3", std::process::id())
    } else {
        "ibkr-agent-gateway-cli.sqlite3".to_string()
    };
    std::env::temp_dir().join(filename)
}

fn running_under_cargo_test() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .is_some_and(|parent| parent.ends_with("deps"))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CliConfigFile {
    broker: BrokerConfigFile,
    auth: AuthConfigFile,
    audit: AuditConfigFile,
    #[serde(default)]
    safety: SafetyConfigFile,
    #[serde(default)]
    live_trading: LiveTradingConfigFile,
    #[serde(default)]
    remote_mcp: RemoteMcpConfigFile,
    #[serde(default)]
    sidecar: SidecarConfigFile,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BrokerConfigFile {
    backend: String,
    base_url: Option<String>,
    #[serde(default)]
    fixture_root: Option<String>,
    #[serde(default)]
    allow_insecure_tls_for_localhost: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthConfigFile {
    enabled_scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditConfigFile {
    sqlite_path: String,
    hmac_secret_env: String,
    #[serde(default)]
    live_write_retention_days: Option<u32>,
    #[serde(default)]
    export_required_before_purge: Option<bool>,
    #[serde(default)]
    immutable_live_events: Option<bool>,
}

impl AuditConfigFile {
    fn audit_retention_config(&self) -> AuditRetentionConfig {
        let defaults = AuditRetentionConfig::default();
        AuditRetentionConfig {
            live_write_retention_days: self
                .live_write_retention_days
                .unwrap_or(defaults.live_write_retention_days),
            export_required_before_purge: self
                .export_required_before_purge
                .unwrap_or(defaults.export_required_before_purge),
            immutable_live_events: self
                .immutable_live_events
                .unwrap_or(defaults.immutable_live_events),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SafetyConfigFile {
    #[serde(default)]
    remote_mcp_enabled: bool,
    #[serde(default)]
    live_trading_enabled: bool,
    #[serde(default)]
    sidecar_enabled: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveTradingConfigFile {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    allowed_accounts: Vec<String>,
    #[serde(default)]
    risk_policy_id: Option<String>,
    #[serde(default)]
    paper_to_live_checklist_acknowledged: bool,
    #[serde(default)]
    reconciler_interval_seconds: Option<u64>,
}

impl LiveTradingConfigFile {
    fn into_live_config(self) -> Result<LiveTradingConfig, GatewayError> {
        let allowed_accounts = self
            .allowed_accounts
            .into_iter()
            .map(|account| {
                AccountId::new(account).ok_or_else(|| {
                    GatewayError::new(
                        ErrorCode::ConfigInvalid,
                        "live_trading.allowed_accounts contains an invalid account id",
                        false,
                        Some("Use valid IBKR account identifiers".to_string()),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(LiveTradingConfig {
            enabled: self.enabled,
            allowed_accounts,
            risk_policy_id: self.risk_policy_id,
            paper_to_live_checklist_acknowledged: self.paper_to_live_checklist_acknowledged,
            reconciler_interval_seconds: self
                .reconciler_interval_seconds
                .unwrap_or_else(default_live_reconciler_interval_seconds)
                .max(1),
        })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteMcpConfigFile {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    bind_address: Option<String>,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default)]
    issuer: Option<String>,
    #[serde(default)]
    jwks_url: Option<String>,
    #[serde(default)]
    metadata_url: Option<String>,
    #[serde(default)]
    audiences: Vec<String>,
    #[serde(default)]
    allowed_scopes: Vec<String>,
    #[serde(default)]
    clock_skew_seconds: Option<u64>,
    #[serde(default)]
    rate_limit_max_requests: Option<u32>,
    #[serde(default)]
    rate_limit_window_seconds: Option<u64>,
    #[serde(default)]
    max_connections: Option<usize>,
    #[serde(default)]
    token_id_hmac_secret_env: Option<String>,
    #[serde(default)]
    token_id_hmac_secret: Option<String>,
}

impl RemoteMcpConfigFile {
    fn into_remote_mcp_config(self) -> Result<RemoteMcpConfig, GatewayError> {
        let token_id_hmac_secret = match (
            self.token_id_hmac_secret_env.as_deref(),
            self.token_id_hmac_secret,
        ) {
            (Some(env_name), _) if !env_name.trim().is_empty() => {
                Some(String::from_utf8(secret_from_env(env_name)?).map_err(|_| {
                    GatewayError::new(
                        ErrorCode::ConfigInvalid,
                        "remote_mcp token HMAC secret must be UTF-8",
                        false,
                        Some("Use a UTF-8 compatible deployment secret".to_string()),
                    )
                })?)
            }
            (_, Some(secret)) if !secret.trim().is_empty() => Some(secret),
            _ => None,
        };

        Ok(RemoteMcpConfig {
            enabled: self.enabled,
            bind_address: self
                .bind_address
                .unwrap_or_else(|| RemoteMcpConfig::default().bind_address),
            resource: parse_optional_url(self.resource, "remote_mcp.resource")?,
            issuer: parse_optional_url(self.issuer, "remote_mcp.issuer")?,
            jwks_url: parse_optional_url(self.jwks_url, "remote_mcp.jwks_url")?,
            metadata_url: parse_optional_url(self.metadata_url, "remote_mcp.metadata_url")?,
            audiences: self.audiences,
            allowed_scopes: self.allowed_scopes,
            clock_skew_seconds: self
                .clock_skew_seconds
                .unwrap_or_else(|| RemoteMcpConfig::default().clock_skew_seconds),
            rate_limit_max_requests: self
                .rate_limit_max_requests
                .unwrap_or_else(|| RemoteMcpConfig::default().rate_limit_max_requests),
            rate_limit_window_seconds: self
                .rate_limit_window_seconds
                .unwrap_or_else(|| RemoteMcpConfig::default().rate_limit_window_seconds),
            max_connections: self
                .max_connections
                .unwrap_or_else(|| RemoteMcpConfig::default().max_connections),
            token_id_hmac_secret,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SidecarConfigFile {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    remote_relay_url: Option<String>,
    #[serde(default)]
    local_client_portal_base_url: Option<String>,
    #[serde(default)]
    heartbeat_interval_seconds: Option<u64>,
    #[serde(default)]
    heartbeat_timeout_seconds: Option<u64>,
}

impl SidecarConfigFile {
    fn into_sidecar_config(self) -> Result<SidecarConfig, GatewayError> {
        let defaults = SidecarConfig::default();
        Ok(SidecarConfig {
            enabled: self.enabled,
            remote_relay_url: parse_optional_url(
                self.remote_relay_url,
                "sidecar.remote_relay_url",
            )?,
            local_client_portal_base_url: parse_optional_url(
                self.local_client_portal_base_url,
                "sidecar.local_client_portal_base_url",
            )?,
            heartbeat_interval_seconds: self
                .heartbeat_interval_seconds
                .unwrap_or(defaults.heartbeat_interval_seconds),
            heartbeat_timeout_seconds: self
                .heartbeat_timeout_seconds
                .unwrap_or(defaults.heartbeat_timeout_seconds),
        })
    }
}

fn parse_backend(value: &str) -> Result<BrokerBackendKind, GatewayError> {
    match value {
        "fake" => Ok(BrokerBackendKind::Fake),
        "client_portal_gateway" => Ok(BrokerBackendKind::ClientPortalGateway),
        _ => Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            format!("Unsupported broker backend: {value}"),
            false,
            Some("Use broker.backend: fake or client_portal_gateway".to_string()),
        )),
    }
}

fn parse_url(value: Option<&str>, field: &str) -> Result<Url, GatewayError> {
    let Some(value) = value else {
        return Err(GatewayError::new(
            ErrorCode::ConfigMissingBrokerBaseUrl,
            format!("{field} is required for Client Portal Gateway"),
            false,
            Some("Configure broker.base_url".to_string()),
        ));
    };
    Url::parse(value).map_err(|_| {
        GatewayError::new(
            ErrorCode::ConfigInvalid,
            format!("{field} must be a valid URL"),
            false,
            Some("Use a valid Client Portal Gateway URL".to_string()),
        )
    })
}

fn parse_optional_url(value: Option<String>, field: &str) -> Result<Option<Url>, GatewayError> {
    value
        .map(|value| {
            Url::parse(&value).map_err(|_| {
                GatewayError::new(
                    ErrorCode::ConfigInvalid,
                    format!("{field} must be a valid URL"),
                    false,
                    Some(format!("Configure a valid URL for {field}")),
                )
            })
        })
        .transpose()
}

fn secret_from_env(env_name: &str) -> Result<Vec<u8>, GatewayError> {
    let value = std::env::var(env_name).map_err(|_| {
        GatewayError::new(
            ErrorCode::ConfigInvalid,
            format!("Required secret env var is not set: {env_name}"),
            false,
            Some(format!("Set {env_name} before using this config")),
        )
    })?;
    Ok(value.into_bytes())
}

fn sqlite_url_from_config_path(value: &str) -> Result<String, GatewayError> {
    if value.starts_with("sqlite:") {
        create_sqlite_parent_if_needed(value)?;
        Ok(value.to_string())
    } else {
        sqlite_url_from_path(Path::new(value))
    }
}

fn sqlite_url_from_path(path: &Path) -> Result<String, GatewayError> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            GatewayError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Unable to create audit directory {}: {error}",
                    parent.display()
                ),
                true,
                Some("Create the audit directory or change audit.sqlite_path".to_string()),
            )
        })?;
    }
    Ok(format!("sqlite://{}?mode=rwc", path.display()))
}

fn create_sqlite_parent_if_needed(database_url: &str) -> Result<(), GatewayError> {
    if database_url == "sqlite::memory:" {
        return Ok(());
    }
    let path = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"));
    let Some(path) = path else {
        return Ok(());
    };
    if path.is_empty() || path == ":memory:" {
        return Ok(());
    }
    let path = Path::new(path);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            GatewayError::new(
                ErrorCode::ConfigInvalid,
                format!(
                    "Unable to create audit directory {}: {error}",
                    parent.display()
                ),
                true,
                Some("Create the audit directory or change audit.sqlite_path".to_string()),
            )
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AuditConfigFile, CliConfigFile, LiveTradingConfigFile, RemoteMcpConfigFile,
        SidecarConfigFile, parse_url,
    };
    use crate::internal::config::{
        validate_audit_retention_config, validate_sidecar_config,
        validate_tls_bypass_localhost_only,
    };
    use crate::internal::domain::{AccountId, ErrorCode};

    #[test]
    fn live_config_file_preserves_operator_allowlist() -> Result<(), Box<dyn std::error::Error>> {
        let config = LiveTradingConfigFile {
            enabled: true,
            allowed_accounts: vec!["U1111111".to_string()],
            risk_policy_id: Some("configured-policy".to_string()),
            paper_to_live_checklist_acknowledged: true,
            reconciler_interval_seconds: Some(7),
        }
        .into_live_config()?;

        assert_eq!(
            config.allowed_accounts,
            vec![AccountId::from_static("U1111111")]
        );
        assert_eq!(config.risk_policy_id.as_deref(), Some("configured-policy"));
        assert_eq!(config.reconciler_interval_seconds, 7);
        Ok(())
    }

    #[test]
    fn remote_mcp_config_file_accepts_complete_direct_secret_config()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = RemoteMcpConfigFile {
            enabled: true,
            bind_address: Some("0.0.0.0:8080".to_string()),
            resource: Some("https://gateway.example.com/mcp".to_string()),
            issuer: Some("https://auth.example.com/".to_string()),
            jwks_url: Some("https://auth.example.com/.well-known/jwks.json".to_string()),
            metadata_url: Some(
                "https://auth.example.com/.well-known/openid-configuration".to_string(),
            ),
            audiences: vec!["https://gateway.example.com/mcp".to_string()],
            allowed_scopes: vec!["ibkr:health:read".to_string()],
            clock_skew_seconds: Some(60),
            rate_limit_max_requests: Some(10),
            rate_limit_window_seconds: Some(30),
            max_connections: Some(8),
            token_id_hmac_secret_env: None,
            token_id_hmac_secret: Some("remote-token-hmac-secret".to_string()),
        }
        .into_remote_mcp_config()?;

        assert!(config.enabled);
        assert_eq!(config.bind_address, "0.0.0.0:8080");
        assert_eq!(
            config.token_id_hmac_secret.as_deref(),
            Some("remote-token-hmac-secret")
        );
        assert_eq!(config.rate_limit_max_requests, 10);
        assert_eq!(config.rate_limit_window_seconds, 30);
        assert_eq!(config.max_connections, 8);
        Ok(())
    }

    #[test]
    fn cli_audit_retention_config_rejects_short_live_retention()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = AuditConfigFile {
            sqlite_path: "./data/audit.sqlite3".to_string(),
            hmac_secret_env: "IBKR_AUDIT_HMAC_SECRET".to_string(),
            live_write_retention_days: Some(30),
            export_required_before_purge: Some(true),
            immutable_live_events: Some(true),
        }
        .audit_retention_config();

        let Err(error) = validate_audit_retention_config(&config, true) else {
            return Err("short live retention should be rejected".into());
        };

        assert_eq!(error.code, ErrorCode::AuditWriteFailed);
        Ok(())
    }

    #[test]
    fn cli_config_rejects_unknown_safety_fields() -> Result<(), Box<dyn std::error::Error>> {
        let yaml = r#"
broker:
  backend: fake
auth:
  enabled_scopes:
    - ibkr:health:read
audit:
  sqlite_path: ./data/audit.sqlite3
  hmac_secret_env: IBKR_AUDIT_HMAC_SECRET
safety:
  read_only: true
"#;

        let parsed = config::Config::builder()
            .add_source(config::File::from_str(yaml, config::FileFormat::Yaml))
            .build()
            .and_then(config::Config::try_deserialize::<CliConfigFile>);

        if parsed.is_ok() {
            return Err("unknown safety fields should be rejected".into());
        }
        Ok(())
    }

    #[test]
    fn cli_sidecar_config_file_rejects_safety_flag_without_enabled_sidecar()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = SidecarConfigFile::default().into_sidecar_config()?;
        let Err(error) = validate_sidecar_config(&config, true, true) else {
            return Err("safety.sidecar_enabled without sidecar.enabled must be rejected".into());
        };
        assert_eq!(error.code, ErrorCode::ConfigSidecarForbidden);
        Ok(())
    }

    #[test]
    fn cli_sidecar_config_file_accepts_complete_enabled_sidecar()
    -> Result<(), Box<dyn std::error::Error>> {
        let config = SidecarConfigFile {
            enabled: true,
            remote_relay_url: Some("https://relay.example.com".to_string()),
            local_client_portal_base_url: Some("https://localhost:5000".to_string()),
            heartbeat_interval_seconds: Some(10),
            heartbeat_timeout_seconds: Some(30),
        }
        .into_sidecar_config()?;

        validate_sidecar_config(&config, true, true)?;
        Ok(())
    }

    #[test]
    fn cli_runtime_reuses_tls_bypass_localhost_validation() -> Result<(), Box<dyn std::error::Error>>
    {
        let remote_url = parse_url(Some("https://broker.example.com"), "broker.base_url")?;
        let error = validate_tls_bypass_localhost_only(&remote_url, false);
        let Err(error) = error else {
            return Err("remote TLS bypass must be rejected".into());
        };

        assert_eq!(error.code, ErrorCode::ConfigTlsBypassNonLocalhost);
        Ok(())
    }
}
