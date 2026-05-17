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
        }
    }

    /// Overrides TLS verification for Client Portal Gateway calls.
    #[must_use]
    pub const fn with_verify_tls(mut self, verify_tls: bool) -> Self {
        self.verify_tls = verify_tls;
        self
    }

    fn backend_factory_config(&self) -> BackendFactoryConfig {
        BackendFactoryConfig {
            backend: self.backend,
            fixture_root: self.fixture_root.clone(),
            client_portal_base_url: self.client_portal_base_url.clone(),
            verify_tls: self.verify_tls,
        }
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
    pub fn new(config: GatewayConfig) -> Result<Self, GatewayError> {
        let backend = create_backend(config.backend_factory_config())?.into();
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
