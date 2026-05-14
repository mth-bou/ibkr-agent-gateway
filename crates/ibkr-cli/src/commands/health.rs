//! Health command.

use crate::audit::build_cli_audit_event;
use crate::output::print_output;
use ibkr_audit::{AuditEventType, AuditResultStatus};
use ibkr_auth::HEALTH_READ;
use ibkr_domain::GatewayError;
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
