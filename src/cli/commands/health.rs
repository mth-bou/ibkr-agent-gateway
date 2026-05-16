//! Health command.

use crate::cli::audit::build_cli_audit_event;
use crate::cli::output::print_output;
use crate::internal::audit::{AuditEventType, AuditResultStatus};
use crate::internal::auth::HEALTH_READ;
use crate::internal::domain::GatewayError;
use serde::Serialize;

/// Health command output.
#[derive(Debug, Serialize)]
pub struct HealthOutput {
    /// Gateway status.
    pub gateway: &'static str,
    /// Read-only mode marker.
    pub read_only: bool,
}

/// Runs `ibkr-agent health`.
pub fn run(json: bool) -> Result<(), GatewayError> {
    let output = HealthOutput {
        gateway: "ok",
        read_only: true,
    };
    let _event = build_cli_audit_event(
        "ibkr_health",
        HEALTH_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "gateway: ok\nmode: read-only", &output)
}
