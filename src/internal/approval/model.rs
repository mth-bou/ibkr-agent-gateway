//! Approval record models.

use ibkr_domain::{AccountId, LocalUserId, OrderPreviewId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Approval identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalId(Uuid);

impl ApprovalId {
    /// Creates a fresh approval id.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Returns the inner UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for ApprovalId {
    fn default() -> Self {
        Self::new()
    }
}

/// Approval lifecycle status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// Approval exists but is not usable yet.
    Pending,
    /// Approval can authorize one paper submit.
    Approved,
    /// Approval was consumed by a submit.
    Consumed,
    /// Approval expired.
    Expired,
    /// Approval was revoked.
    Revoked,
}

/// Explicit approval record for a preview.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRecord {
    /// Approval id.
    pub approval_id: ApprovalId,
    /// Approved preview id.
    pub preview_id: OrderPreviewId,
    /// Paper account id.
    pub account_id: AccountId,
    /// User who approved.
    pub approved_by: LocalUserId,
    /// Approval status.
    pub status: ApprovalStatus,
    /// Approval timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    #[schemars(with = "Option<String>")]
    pub approved_at: Option<OffsetDateTime>,
    /// Expiration timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub expires_at: OffsetDateTime,
}
