//! Approval commands.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use crate::internal::approval::ApprovalService;
use crate::internal::audit::SqliteAuditWriter;
use crate::internal::domain::{ErrorCode, GatewayError, LocalUserId, OrderPreviewId};
use time::OffsetDateTime;

/// Creates a local approval record for a paper preview.
pub async fn create(
    audit_writer: &SqliteAuditWriter,
    account: &str,
    preview_id: &str,
    ttl_seconds: i64,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let preview_id = OrderPreviewId::parse(preview_id)?;
    let preview_record = audit_writer
        .load_order_preview(&preview_id)
        .await?
        .ok_or_else(missing_preview)?;
    if preview_record.validated_order.account_id != account_id {
        return Err(GatewayError::new(
            ErrorCode::ApprovalPreviewMismatch,
            "Approval account does not match the preview account",
            false,
            Some("Create the approval for the account used by the preview".to_string()),
        ));
    }
    if preview_record.preview.expires_at <= OffsetDateTime::now_utc() {
        return Err(GatewayError::new(
            ErrorCode::PaperApprovalRequired,
            "Cannot approve an expired order preview",
            false,
            Some("Create a fresh preview before approval".to_string()),
        ));
    }

    let mut service = ApprovalService::default();
    let approval = service.create_approval(
        preview_id,
        account_id,
        LocalUserId::from_static("local-user"),
        ttl_seconds,
    );
    audit_writer.append_approval(&approval).await?;
    print_output(json, "approval created", &approval)
}

fn missing_preview() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Approval requires an existing order preview",
        false,
        Some("Run `ibkr-agent orders preview --enable-preview` and pass --preview-id".to_string()),
    )
}
