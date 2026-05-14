//! Optional MCP session keepalive loop helpers.

use ibkr_backend::IbkrBackend;
use ibkr_domain::{BrokerSessionStatus, GatewayError};

/// Runs one keepalive tick.
pub async fn keepalive_once(
    backend: &dyn IbkrBackend,
) -> Result<BrokerSessionStatus, GatewayError> {
    backend.keepalive().await
}
