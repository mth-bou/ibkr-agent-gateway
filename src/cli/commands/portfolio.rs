//! Portfolio snapshot command.

use crate::cli::{
    audit::build_cli_audit_event, commands::account::parse_account_id, output::print_output,
};
use crate::internal::audit::{AuditEventType, AuditResultStatus};
use crate::internal::auth::PORTFOLIO_READ;
use crate::internal::backend::IbkrBackend;
use crate::internal::domain::GatewayError;

/// Runs `ibkr-agent portfolio snapshot`.
pub async fn snapshot(
    backend: &dyn IbkrBackend,
    account: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let value = backend.portfolio_snapshot(&account_id).await?;
    let _event = build_cli_audit_event(
        "ibkr_portfolio_snapshot",
        PORTFOLIO_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "portfolio snapshot returned", &value)
}
