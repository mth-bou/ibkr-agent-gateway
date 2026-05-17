//! Shared approval-to-preview binding checks.

use crate::internal::approval::{ApprovalRecord, ApprovalStatus};
use crate::internal::domain::{ErrorCode, GatewayError, ValidatedOrder};
use time::OffsetDateTime;

pub(super) fn validate_approved_preview(
    approval: &ApprovalRecord,
    order: &ValidatedOrder,
    now: OffsetDateTime,
    missing_code: ErrorCode,
    missing_message: &'static str,
    missing_action: &'static str,
) -> Result<(), GatewayError> {
    if approval.status == ApprovalStatus::Consumed {
        return Err(GatewayError::new(
            ErrorCode::ApprovalConsumed,
            "Approval was already consumed by a previous submit",
            false,
            Some("Create a new approval for a fresh preview".to_string()),
        ));
    }

    if approval.status != ApprovalStatus::Approved
        || approval.account_id != order.account_id
        || approval.expires_at <= now
    {
        return Err(GatewayError::new(
            missing_code,
            missing_message,
            false,
            Some(missing_action.to_string()),
        ));
    }

    if approval.preview_id != order.preview_id {
        return Err(GatewayError::new(
            ErrorCode::ApprovalPreviewMismatch,
            "Approval preview id does not match the submitted order preview id",
            false,
            Some("Submit the order produced by the approved preview".to_string()),
        ));
    }

    Ok(())
}
