use crate::internal::audit::AuditHmacKey;
use crate::internal::backend::{BackendFactoryConfig, IbkrBackend, create_backend};
use crate::internal::domain::{
    BrokerAccount, BrokerBackendKind, BrokerSessionStatus, ContractCandidate, GatewayError,
};
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// SDK configuration for constructing a gateway client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayConfig {
    /// Broker backend used by the gateway.
    pub backend: BrokerBackendKind,
    /// Fixture root used when `backend` is `BrokerBackendKind::Fake`.
    pub fixture_root: PathBuf,
    /// Client Portal Gateway base URL used when the real backend is selected.
    pub client_portal_base_url: Option<Url>,
    /// Whether TLS certificates are verified for Client Portal Gateway calls.
    pub verify_tls: bool,
    /// Optional override for the audit HMAC key. When `None`, [`Gateway::new`]
    /// generates a fresh ephemeral key for the process lifetime.
    pub audit_hmac_key: Option<Arc<AuditHmacKey>>,
}

impl GatewayConfig {
    /// Creates an offline fake backend configuration rooted at `tests/fixtures/cpapi`.
    #[must_use]
    pub fn fake_local() -> Self {
        Self::fake_with_fixture_root("tests/fixtures/cpapi")
    }

    /// Creates an offline fake backend configuration with a custom fixture root.
    #[must_use]
    pub fn fake_with_fixture_root(fixture_root: impl Into<PathBuf>) -> Self {
        Self {
            backend: BrokerBackendKind::Fake,
            fixture_root: fixture_root.into(),
            client_portal_base_url: None,
            verify_tls: true,
            audit_hmac_key: None,
        }
    }

    /// Creates a Client Portal Gateway backend configuration.
    #[must_use]
    pub fn client_portal(base_url: Url) -> Self {
        Self {
            backend: BrokerBackendKind::ClientPortalGateway,
            fixture_root: PathBuf::new(),
            client_portal_base_url: Some(base_url),
            verify_tls: true,
            audit_hmac_key: None,
        }
    }

    /// Overrides TLS verification for Client Portal Gateway calls.
    #[must_use]
    pub const fn with_verify_tls(mut self, verify_tls: bool) -> Self {
        self.verify_tls = verify_tls;
        self
    }

    /// Supplies the audit HMAC key used to redact account identifiers.
    ///
    /// When unset, [`Gateway::new`] derives a fresh ephemeral key per process.
    #[must_use]
    pub fn with_audit_hmac_key(mut self, key: AuditHmacKey) -> Self {
        self.audit_hmac_key = Some(Arc::new(key));
        self
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self::fake_local()
    }
}

/// Embeddable gateway client for read-only broker workflows.
#[derive(Clone)]
pub struct Gateway {
    config: GatewayConfig,
    backend: Arc<dyn IbkrBackend>,
}

impl fmt::Debug for Gateway {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Gateway")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Gateway {
    /// Creates a gateway client and validates the selected backend configuration.
    pub fn new(mut config: GatewayConfig) -> Result<Self, GatewayError> {
        let audit_hmac_key = match config.audit_hmac_key.clone() {
            Some(key) => key,
            None => {
                let key = Arc::new(AuditHmacKey::ephemeral()?);
                config.audit_hmac_key = Some(key.clone());
                key
            }
        };
        let factory_config = BackendFactoryConfig {
            backend: config.backend,
            fixture_root: config.fixture_root.clone(),
            client_portal_base_url: config.client_portal_base_url.clone(),
            verify_tls: config.verify_tls,
            audit_hmac_key,
        };
        let backend = create_backend(factory_config)?.into();
        Ok(Self { config, backend })
    }

    /// Returns the configuration used by this gateway.
    #[must_use]
    pub const fn config(&self) -> &GatewayConfig {
        &self.config
    }

    /// Returns broker session status without exposing secrets.
    pub async fn session_status(&self) -> Result<BrokerSessionStatus, GatewayError> {
        self.backend.session_status().await
    }

    /// Attempts a broker keepalive and returns the resulting session status.
    pub async fn keepalive(&self) -> Result<BrokerSessionStatus, GatewayError> {
        self.backend.keepalive().await
    }

    /// Lists accounts visible to the configured broker session.
    pub async fn list_accounts(&self) -> Result<Vec<BrokerAccount>, GatewayError> {
        self.backend.list_accounts().await
    }

    /// Searches contract candidates using the configured backend.
    pub async fn search_contracts(
        &self,
        query: impl AsRef<str>,
    ) -> Result<Vec<ContractCandidate>, GatewayError> {
        self.backend.search_contracts(query.as_ref()).await
    }
}
