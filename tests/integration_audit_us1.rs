use ibkr_agent_gateway::testing::audit::{AuditEventType, AuditResultStatus};
use ibkr_agent_gateway::testing::auth::HEALTH_READ;

#[test]
fn cli_us1_audit_event_uses_safe_scope_and_tool_name() {
    let event = ibkr_agent_gateway::cli::audit::build_cli_audit_event(
        "ibkr_health",
        HEALTH_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );

    assert_eq!(event.tool_name.as_deref(), Some("ibkr_health"));
    assert_eq!(event.scopes, vec![HEALTH_READ.to_string()]);
    assert!(event.redactions.is_empty());
}
