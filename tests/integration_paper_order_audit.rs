use ibkr_agent_gateway::testing::audit::{AuditDecision, AuditEventType, AuditResultStatus};
use ibkr_agent_gateway::testing::orders::audit::build_paper_order_audit_event;

#[test]
fn paper_order_audit_events_use_paper_tools_and_scopes() {
    let submitted = build_paper_order_audit_event(
        "ibkr_paper_order_submit",
        "ibkr:orders:paper:submit",
        AuditEventType::PaperOrderSubmitted,
        AuditResultStatus::Completed,
    );

    assert_eq!(
        submitted.tool_name.as_deref(),
        Some("ibkr_paper_order_submit")
    );
    assert_eq!(submitted.scopes, vec!["ibkr:orders:paper:submit"]);
    assert_eq!(submitted.decision, AuditDecision::Allow);

    let cancelled = build_paper_order_audit_event(
        "ibkr_paper_order_cancel",
        "ibkr:orders:paper:cancel",
        AuditEventType::PaperOrderCancelled,
        AuditResultStatus::Refused,
    );

    assert_eq!(
        cancelled.tool_name.as_deref(),
        Some("ibkr_paper_order_cancel")
    );
    assert_eq!(cancelled.decision, AuditDecision::Refuse);
}
