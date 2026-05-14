//! Account summary command.

use crate::{audit::build_cli_audit_event, output::print_output};
use ibkr_audit::{AuditEventType, AuditResultStatus};
use ibkr_auth::PORTFOLIO_READ;
use ibkr_backend::IbkrBackend;
use ibkr_domain::{AccountId, ErrorCode, GatewayError};

/// Runs `ibkr-agent account summary`.
pub async fn summary(
    backend: &dyn IbkrBackend,
    account: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let value = backend.account_summary(&account_id).await?;
    let _event = build_cli_audit_event(
        "ibkr_account_summary",
        PORTFOLIO_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "account summary returned", &value)
}

pub(crate) fn parse_account_id(account: &str) -> Result<AccountId, GatewayError> {
    AccountId::new(account).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::InputMissingAccount,
            "Account id is required",
            false,
            Some("Select one account explicitly".to_string()),
        )
    })
}
