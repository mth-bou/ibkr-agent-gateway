//! Positions command.

use crate::{
    audit::build_cli_audit_event, commands::account::parse_account_id, output::print_output,
};
use ibkr_audit::{AuditEventType, AuditResultStatus};
use ibkr_auth::POSITIONS_READ;
use ibkr_backend::IbkrBackend;
use ibkr_domain::GatewayError;

/// Runs `ibkr-agent positions list`.
pub async fn list(
    backend: &dyn IbkrBackend,
    account: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let value = backend.positions(&account_id).await?;
    let _event = build_cli_audit_event(
        "ibkr_positions_list",
        POSITIONS_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "positions returned", &value)
}
