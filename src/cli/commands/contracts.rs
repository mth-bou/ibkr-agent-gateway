//! Contract search and resolution commands.

use crate::cli::{audit::build_cli_audit_event, output::print_output};
use crate::internal::audit::{AuditEventType, AuditResultStatus};
use crate::internal::auth::MARKETDATA_READ;
use crate::internal::backend::IbkrBackend;
use crate::internal::domain::GatewayError;

/// Runs `ibkr-agent contracts search`.
pub async fn search(
    backend: &dyn IbkrBackend,
    query: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let value = backend.search_contracts(query).await?;
    let _event = build_cli_audit_event(
        "ibkr_contracts_search",
        MARKETDATA_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "contract candidates returned", &value)
}

/// Runs `ibkr-agent contracts resolve`.
pub async fn resolve(
    backend: &dyn IbkrBackend,
    query: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let value = backend.resolve_contract(query).await?;
    let _event = build_cli_audit_event(
        "ibkr_contract_resolve",
        MARKETDATA_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "contract resolved", &value)
}
