//! Sidecar heartbeat handling.

use super::{RelaySession, RelaySessionId, SidecarId};
use ibkr_domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Heartbeat sent by a paired sidecar.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SidecarHeartbeat {
    /// Sidecar id.
    pub sidecar_id: SidecarId,
    /// Relay session id.
    pub relay_session_id: RelaySessionId,
    /// Sidecar observed timestamp.
    #[schemars(with = "String")]
    #[serde(with = "time::serde::rfc3339")]
    pub observed_at: OffsetDateTime,
}

/// Applies a heartbeat to an existing session after binding checks.
pub fn apply_heartbeat(
    session: &mut RelaySession,
    heartbeat: &SidecarHeartbeat,
) -> Result<(), GatewayError> {
    if session.sidecar_id != heartbeat.sidecar_id
        || session.relay_session_id != heartbeat.relay_session_id
    {
        return Err(GatewayError::new(
            ErrorCode::SidecarSessionInvalid,
            "Sidecar heartbeat does not match the relay session binding",
            false,
            Some("Re-pair the sidecar before forwarding broker requests".to_string()),
        ));
    }
    session.heartbeat_at = heartbeat.observed_at;
    Ok(())
}
