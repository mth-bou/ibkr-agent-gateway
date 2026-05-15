//! Remote gateway sidecar relay endpoint.

use ibkr_domain::GatewayError;
use ibkr_sidecar::{
    ForwardedBrokerRequest, RelaySession, build_forwarded_broker_request, require_available_session,
};
use time::OffsetDateTime;

/// Result of accepting a sidecar relay request.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SidecarRelayAccepted {
    /// Sanitized broker request.
    pub forwarded_request: ForwardedBrokerRequest,
}

/// Validates relay session state and builds a sanitized forward request.
pub fn accept_sidecar_relay_request(
    session: Option<&RelaySession>,
    tool_name: &str,
    scope: &str,
    payload: &serde_json::Value,
    now: OffsetDateTime,
) -> Result<SidecarRelayAccepted, GatewayError> {
    require_available_session(session, now)?;
    let forwarded_request = build_forwarded_broker_request(tool_name, scope, payload)?;
    Ok(SidecarRelayAccepted { forwarded_request })
}
