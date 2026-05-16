//! Stable sidecar identity.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Sidecar identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct SidecarId(String);

impl SidecarId {
    /// Creates a fresh sidecar id.
    #[must_use]
    pub fn new() -> Self {
        Self(format!("sidecar-{}", Uuid::now_v7()))
    }

    /// Creates a sidecar id from a non-empty string.
    #[must_use]
    pub fn from_string(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Returns the raw identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SidecarId {
    fn default() -> Self {
        Self::new()
    }
}

/// Sidecar capability advertised to the remote gateway.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SidecarCapability {
    /// Read-only broker requests.
    BrokerRead,
    /// Paper trading requests, gated by previous specs and scopes.
    PaperTrading,
    /// Heartbeat/session health reporting.
    Heartbeat,
}

/// Public sidecar identity. Private key material stays local and is never stored here.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SidecarIdentity {
    /// Stable sidecar id.
    pub sidecar_id: SidecarId,
    /// Public key or public key fingerprint.
    pub public_key: String,
    /// Creation timestamp.
    #[schemars(with = "String")]
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Operator-facing display name.
    pub display_name: Option<String>,
    /// Advertised capabilities.
    pub capabilities: Vec<SidecarCapability>,
}

impl SidecarIdentity {
    /// Creates a new identity from a public key/fingerprint.
    #[must_use]
    pub fn new(public_key: impl Into<String>, display_name: Option<String>) -> Self {
        Self {
            sidecar_id: SidecarId::new(),
            public_key: public_key.into(),
            created_at: OffsetDateTime::now_utc(),
            display_name,
            capabilities: vec![SidecarCapability::BrokerRead, SidecarCapability::Heartbeat],
        }
    }
}
