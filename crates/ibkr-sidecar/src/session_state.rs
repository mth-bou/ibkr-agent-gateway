//! Fail-closed relay session availability checks.

use crate::RelaySession;
use ibkr_domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Relay availability decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelayAvailability {
    /// Relay session can forward broker requests.
    Available,
    /// Relay session is disconnected or missing.
    Disconnected,
    /// Relay session expired.
    Expired,
}

/// Requires a relay session to be available before broker forwarding.
pub fn require_available_session(
    session: Option<&RelaySession>,
    now: OffsetDateTime,
) -> Result<RelayAvailability, GatewayError> {
    let Some(session) = session else {
        return Err(unavailable("No sidecar relay session is bound"));
    };
    if session.expires_at <= now {
        return Err(GatewayError::new(
            ErrorCode::SidecarSessionInvalid,
            "Sidecar relay session is expired",
            false,
            Some("Re-establish the sidecar relay session".to_string()),
        ));
    }
    if session.heartbeat_at > now {
        return Err(unavailable("Sidecar heartbeat timestamp is invalid"));
    }

    Ok(RelayAvailability::Available)
}

fn unavailable(message: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::SidecarUnavailable,
        message,
        true,
        Some("Check the local sidecar and Client Portal Gateway session".to_string()),
    )
}
