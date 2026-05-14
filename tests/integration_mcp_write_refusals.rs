use ibkr_domain::ErrorCode;
use ibkr_mcp::{FORBIDDEN_TOOL_NAMES, is_forbidden_tool_name, refuse_forbidden_tool};

#[test]
fn forbidden_write_tools_are_absent_and_refused() {
    for tool in FORBIDDEN_TOOL_NAMES {
        assert!(is_forbidden_tool_name(tool));
        let error = refuse_forbidden_tool(tool);
        assert_eq!(error.code, ErrorCode::ReadonlyWriteForbidden);
    }
}

#[test]
fn mcp_refused_calls_have_refused_audit_event() {
    let event = ibkr_mcp::build_mcp_tool_event(
        "ibkr_order_submit",
        "ibkr:orders:read",
        ibkr_audit::AuditResultStatus::Refused,
    );

    assert_eq!(event.event_type, ibkr_audit::AuditEventType::ToolRefused);
    assert_eq!(event.decision, ibkr_audit::AuditDecision::Refuse);
}
