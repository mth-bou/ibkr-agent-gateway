//! Approval creation and lookup service.

use super::model::{ApprovalId, ApprovalRecord, ApprovalStatus};
use crate::internal::domain::{AccountId, LocalUserId, OrderPreviewId};
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime};

const MIN_APPROVAL_TTL_SECONDS: i64 = 60;
const MAX_APPROVAL_TTL_SECONDS: i64 = 3_600;

/// In-memory approval service for local paper workflows.
#[derive(Clone, Debug, Default)]
pub struct ApprovalService {
    approvals: BTreeMap<String, ApprovalRecord>,
}

impl ApprovalService {
    /// Creates an approved record for one preview.
    #[must_use]
    pub fn create_approval(
        &mut self,
        preview_id: OrderPreviewId,
        account_id: AccountId,
        approved_by: LocalUserId,
        ttl_seconds: i64,
    ) -> ApprovalRecord {
        let ttl_seconds = bounded_ttl_seconds(ttl_seconds);
        let approval = ApprovalRecord {
            approval_id: ApprovalId::new(),
            preview_id,
            account_id,
            approved_by,
            status: ApprovalStatus::Approved,
            approved_at: Some(OffsetDateTime::now_utc()),
            expires_at: OffsetDateTime::now_utc() + Duration::seconds(ttl_seconds),
        };
        self.approvals
            .insert(approval.approval_id.as_uuid().to_string(), approval.clone());
        approval
    }

    /// Reads one approval by id.
    #[must_use]
    pub fn get(&self, approval_id: &ApprovalId) -> Option<&ApprovalRecord> {
        self.approvals.get(&approval_id.as_uuid().to_string())
    }
}

const fn bounded_ttl_seconds(ttl_seconds: i64) -> i64 {
    if ttl_seconds < MIN_APPROVAL_TTL_SECONDS {
        MIN_APPROVAL_TTL_SECONDS
    } else if ttl_seconds > MAX_APPROVAL_TTL_SECONDS {
        MAX_APPROVAL_TTL_SECONDS
    } else {
        ttl_seconds
    }
}
