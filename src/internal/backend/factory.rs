//! Backend factory.

use super::{ClientPortalBackend, FakeBackend, FakeFixtureStore, IbkrBackend};
use crate::internal::audit::AuditHmacKey;
use crate::internal::cpapi::ClientPortalClient;
use crate::internal::domain::{BrokerBackendKind, ErrorCode, GatewayError};
use std::path::PathBuf;
use std::sync::Arc;
use url::Url;

/// Backend factory input.
#[derive(Clone, Debug)]
pub struct BackendFactoryConfig {
    /// Backend kind.
    pub backend: BrokerBackendKind,
    /// Fixture root for fake backend.
    pub fixture_root: PathBuf,
    /// Client Portal Gateway base URL.
    pub client_portal_base_url: Option<Url>,
    /// Whether TLS is verified.
    pub verify_tls: bool,
    /// Key used to HMAC account identifiers in audit records.
    pub audit_hmac_key: Arc<AuditHmacKey>,
}

/// Creates a boxed backend from configuration.
pub fn create_backend(config: BackendFactoryConfig) -> Result<Box<dyn IbkrBackend>, GatewayError> {
    match config.backend {
        BrokerBackendKind::Fake => Ok(Box::new(FakeBackend::new(FakeFixtureStore::new(
            config.fixture_root,
        )))),
        BrokerBackendKind::ClientPortalGateway => {
            let Some(base_url) = config.client_portal_base_url else {
                return Err(GatewayError::new(
                    ErrorCode::ConfigMissingBrokerBaseUrl,
                    "Client Portal Gateway base URL is required",
                    false,
                    Some("Configure broker.client_portal_gateway.base_url".to_string()),
                ));
            };
            Ok(Box::new(ClientPortalBackend::new(
                ClientPortalClient::new(base_url, config.verify_tls)?,
                config.audit_hmac_key,
            )))
        }
    }
}
