//! Account listing command.

use crate::cli::audit::build_cli_audit_event;
use crate::cli::output::print_output;
use crate::internal::audit::{AuditEventType, AuditResultStatus};
use crate::internal::auth::ACCOUNTS_READ;
use crate::internal::backend::IbkrBackend;
use crate::internal::domain::GatewayError;

/// Runs `ibkr-agent accounts list`.
pub async fn list(backend: &dyn IbkrBackend, json: bool) -> Result<(), GatewayError> {
    let accounts = backend.list_accounts().await?;
    let _event = build_cli_audit_event(
        "ibkr_accounts_list",
        ACCOUNTS_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    if json {
        return print_output(true, "", &accounts);
    }
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
                    .map_or("unknown", crate::internal::domain::CurrencyCode::as_str)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    print_output(json, &human, &accounts)
}
