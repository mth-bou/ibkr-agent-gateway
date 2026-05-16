//! Account listing command.

use crate::cli::audit::build_cli_audit_event;
use crate::cli::output::print_output;
use ibkr_audit::{AuditEventType, AuditResultStatus};
use ibkr_auth::ACCOUNTS_READ;
use ibkr_backend::IbkrBackend;
use ibkr_domain::GatewayError;

/// Runs `ibkr-agent accounts list`.
pub async fn list(backend: &dyn IbkrBackend, json: bool) -> Result<(), GatewayError> {
    let accounts = backend.list_accounts().await?;
    let human = accounts
        .iter()
        .map(|account| {
            format!(
                "{} {:?} {}",
                account.account_id.as_str(),
                account.account_mode,
                account
                    .base_currency
                    .as_ref()
                    .map_or("unknown", ibkr_domain::CurrencyCode::as_str)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let _event = build_cli_audit_event(
        "ibkr_accounts_list",
        ACCOUNTS_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, &human, &accounts)
}
