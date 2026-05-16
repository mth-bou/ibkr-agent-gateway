//! Approval creation and lookup service.

use super::model::{ApprovalId, ApprovalRecord, ApprovalStatus};
use crate::internal::domain::{AccountId, LocalUserId, OrderPreviewId};
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime};

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
