//! Explicit sidecar pairing records.

use super::identity::SidecarId;
use ibkr_domain::LocalUserId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// Pairing identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct PairingId(String);

impl PairingId {
    /// Creates a fresh pairing id.
    #[must_use]
    pub fn new() -> Self {
        Self(format!("pairing-{}", Uuid::now_v7()))
    }

    /// Returns the raw identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for PairingId {
    fn default() -> Self {
        Self::new()
    }
}

/// Pairing status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PairingStatus {
    /// Pairing is pending operator confirmation.
    Pending,
    /// Pairing is active.
    Active,
    /// Pairing has been revoked.
    Revoked,
    /// Pairing has expired.
    Expired,
}

/// Pairing between a remote gateway instance and a local sidecar.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PairingRecord {
    /// Pairing id.
    pub pairing_id: PairingId,
    /// Remote gateway instance id.
    pub remote_instance_id: String,
    /// Local sidecar id.
    pub sidecar_id: SidecarId,
    /// User id.
    pub user_id: LocalUserId,
    /// Creation timestamp.
    #[schemars(with = "String")]
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Expiration timestamp.
    #[schemars(with = "String")]
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    /// Pairing status.
    pub status: PairingStatus,
}

/// Creates an active pairing record with a bounded TTL.
#[must_use]
pub fn create_pairing(
    remote_instance_id: impl Into<String>,
    sidecar_id: SidecarId,
    user_id: LocalUserId,
    ttl_seconds: i64,
) -> PairingRecord {
    let created_at = OffsetDateTime::now_utc();
    PairingRecord {
        pairing_id: PairingId::new(),
        remote_instance_id: remote_instance_id.into(),
        sidecar_id,
        user_id,
        created_at,
        expires_at: created_at + Duration::seconds(ttl_seconds.max(1)),
        status: PairingStatus::Active,
    }
}
