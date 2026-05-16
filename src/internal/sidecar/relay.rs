//! Relay session protocol between remote gateway and local sidecar.

use super::identity::{SidecarCapability, SidecarId};
use ibkr_domain::RequestId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Relay session identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct RelaySessionId(String);

impl RelaySessionId {
    /// Creates a fresh relay session id.
    #[must_use]
    pub fn new() -> Self {
        Self(format!("relay-{}", Uuid::now_v7()))
    }

    /// Returns the raw identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for RelaySessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Active relay session state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RelaySession {
    /// Relay session id.
    pub relay_session_id: RelaySessionId,
    /// Sidecar id.
    pub sidecar_id: SidecarId,
    /// Remote gateway instance id.
    pub remote_instance_id: String,
    /// Last heartbeat timestamp.
    #[schemars(with = "String")]
    #[serde(with = "time::serde::rfc3339")]
    pub heartbeat_at: OffsetDateTime,
    /// Session expiration timestamp.
    #[schemars(with = "String")]
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    /// Session capabilities.
    pub capabilities: Vec<SidecarCapability>,
}

impl RelaySession {
    /// Returns true when the session is still valid at `now`.
    #[must_use]
    pub fn is_available(&self, now: OffsetDateTime) -> bool {
        self.expires_at > now && self.heartbeat_at <= now
    }
}

/// Broker request forwarded through the sidecar relay.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ForwardedBrokerRequest {
    /// Request correlation id.
    pub request_id: RequestId,
    /// MCP tool name.
    pub tool_name: String,
    /// Required scope.
    pub scope: String,
    /// Hash of the canonical payload.
    pub payload_hash: String,
    /// Creation timestamp.
    #[schemars(with = "String")]
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

/// Creates a relay session after explicit pairing.
#[must_use]
pub fn create_relay_session(
    sidecar_id: SidecarId,
    remote_instance_id: impl Into<String>,
    ttl_seconds: i64,
) -> RelaySession {
    let now = OffsetDateTime::now_utc();
    RelaySession {
        relay_session_id: RelaySessionId::new(),
        sidecar_id,
        remote_instance_id: remote_instance_id.into(),
        heartbeat_at: now,
        expires_at: now + Duration::seconds(ttl_seconds.max(1)),
        capabilities: vec![SidecarCapability::BrokerRead, SidecarCapability::Heartbeat],
    }
}
