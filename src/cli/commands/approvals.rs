//! Approval commands.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use crate::internal::approval::ApprovalService;
use crate::internal::audit::SqliteAuditWriter;
use crate::internal::domain::{GatewayError, LocalUserId, OrderPreviewId};

/// Creates a local approval record for a paper preview.
pub async fn create(
    audit_writer: &SqliteAuditWriter,
    account: &str,
    ttl_seconds: i64,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let mut service = ApprovalService::default();
    let approval = service.create_approval(
        OrderPreviewId::new(),
        account_id,
        LocalUserId::from_static("local-user"),
        ttl_seconds,
    );
    audit_writer.append_approval(&approval).await?;
    print_output(json, "approval created", &approval)
}
