use ibkr_agent_gateway::testing::audit::{AuditDecision, AuditEventType, AuditResultStatus};

#[test]
fn audit_event_contract_includes_required_shapes() {
    let event = ibkr_agent_gateway::testing::mcp::build_mcp_tool_event(
        "ibkr_audit_tail",
        "ibkr:audit:read",
        AuditResultStatus::Completed,
    );

    assert_eq!(event.event_type, AuditEventType::ToolCompleted);
    assert_eq!(event.tool_name.as_deref(), Some("ibkr_audit_tail"));
    assert_eq!(event.scopes, vec!["ibkr:audit:read"]);
    assert_eq!(event.decision, AuditDecision::Allow);
    assert!(event.account_id_hash.is_none());
    assert!(event.error_code.is_none());
}

#[test]
fn audit_event_contract_includes_refused_event_type() {
    let event = ibkr_agent_gateway::testing::mcp::build_mcp_tool_event(
        "ibkr_order_submit",
        "ibkr:orders:read",
        AuditResultStatus::Refused,
    );

    assert_eq!(event.event_type, AuditEventType::ToolRefused);
    assert_eq!(event.decision, AuditDecision::Refuse);
}
