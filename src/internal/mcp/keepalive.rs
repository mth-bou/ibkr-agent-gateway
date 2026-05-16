//! Optional MCP session keepalive loop helpers.

use crate::internal::backend::IbkrBackend;
use crate::internal::domain::{BrokerSessionStatus, GatewayError};

/// Runs one keepalive tick.
pub async fn keepalive_once(
    backend: &dyn IbkrBackend,
) -> Result<BrokerSessionStatus, GatewayError> {
    backend.keepalive().await
}
