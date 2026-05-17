//! CLI runtime construction from defaults or a config file.

use crate::internal::audit::{AuditHmacKey, SqliteAuditWriter};
use crate::internal::auth::{LOCAL_SCOPES, ScopeSet};
use crate::internal::backend::{BackendFactoryConfig, IbkrBackend, create_backend};
use crate::internal::domain::{BrokerBackendKind, ErrorCode, GatewayError};
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
            fixture_root: config.fixture_root,
            client_portal_base_url: config.client_portal_base_url,
            verify_tls: config.verify_tls,
            audit_hmac_key: audit_hmac_key.clone(),
        })?;
        let audit_writer =
            SqliteAuditWriter::connect(&config.audit_database_url, audit_hmac_key.clone()).await?;

        Ok(Self {
            backend,
            audit_writer,
            audit_hmac_key,
            scopes: config.scopes,
        })
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
        let audit_database_url = sqlite_url_from_config_path(&file_config.audit.sqlite_path)?;
        let audit_hmac_secret = secret_from_env(&file_config.audit.hmac_secret_env)?;
        let scopes = ScopeSet::local_with_live(file_config.auth.enabled_scopes)?;

        Ok(Self {
            backend,
            fixture_root: file_config
                .broker
                .fixture_root
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("tests/fixtures/cpapi")),
            client_portal_base_url,
            verify_tls: !file_config
                .broker
                .allow_insecure_tls_for_localhost
                .unwrap_or(false),
            audit_database_url,
            audit_hmac_secret,
            scopes,
        })
    }
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
struct CliConfigFile {
    broker: BrokerConfigFile,
    auth: AuthConfigFile,
    audit: AuditConfigFile,
}

#[derive(Debug, Deserialize)]
struct BrokerConfigFile {
    backend: String,
    base_url: Option<String>,
    #[serde(default)]
    fixture_root: Option<String>,
    #[serde(default)]
    allow_insecure_tls_for_localhost: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AuthConfigFile {
    enabled_scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AuditConfigFile {
    sqlite_path: String,
    hmac_secret_env: String,
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
