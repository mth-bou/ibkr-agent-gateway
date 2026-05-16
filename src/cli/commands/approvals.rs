//! Approval commands.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use ibkr_approval::ApprovalService;
use ibkr_domain::{GatewayError, LocalUserId, OrderPreviewId};

/// Creates a local approval record for a paper preview.
pub fn create(account: &str, ttl_seconds: i64, json: bool) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let mut service = ApprovalService::default();
    let approval = service.create_approval(
        OrderPreviewId::new(),
        account_id,
        LocalUserId::from_static("local-user"),
        ttl_seconds,
    );
    print_output(json, "approval created", &approval)
}
