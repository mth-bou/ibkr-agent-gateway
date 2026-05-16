//! Backend status and session commands.

use crate::cli::audit::build_cli_audit_event;
use crate::cli::output::print_output;
use crate::internal::audit::{AuditEventType, AuditResultStatus};
use crate::internal::auth::HEALTH_READ;
use crate::internal::backend::IbkrBackend;
use crate::internal::domain::{BrokerSessionVisibility, GatewayError};

/// Runs `ibkr-agent backend status`.
pub async fn status(backend: &dyn IbkrBackend, json: bool) -> Result<(), GatewayError> {
    let status = backend.session_status().await?;
    let human = format!("backend: {:?}\nstatus: {:?}", status.backend, status.status);
    let _event = build_cli_audit_event(
        "ibkr_backend_status",
        HEALTH_READ,
        AuditEventType::BackendSessionChecked,
        AuditResultStatus::Completed,
    );
    print_output(json, &human, &status)
}

/// Runs `ibkr-agent session requirements`.
pub async fn requirements(backend: &dyn IbkrBackend, json: bool) -> Result<(), GatewayError> {
    let status = backend.session_status().await?;
    let human = match status.status {
        BrokerSessionVisibility::Usable => "session: usable\nmanual_action: none".to_string(),
        BrokerSessionVisibility::ManualActionRequired | BrokerSessionVisibility::Unavailable => {
            format!(
                "session: {:?}\nmanual_action: {}",
                status.status,
                status
                    .user_action
                    .as_deref()
                    .unwrap_or("Complete broker login manually")
            )
        }
    };
    let _event = build_cli_audit_event(
        "ibkr_session_requirements",
        HEALTH_READ,
        AuditEventType::BackendSessionChecked,
        AuditResultStatus::Completed,
    );
    print_output(json, &human, &status)
}
