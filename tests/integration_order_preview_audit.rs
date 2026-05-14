use ibkr_audit::{AuditDecision, AuditEventType, AuditResultStatus};

#[test]
fn order_preview_audit_success_and_refusal_shapes() {
    let created = ibkr_orders::build_order_audit_event(
        AuditEventType::OrderPreviewCreated,
        AuditResultStatus::Completed,
    );
    assert_eq!(created.tool_name.as_deref(), Some("ibkr_order_preview"));
    assert_eq!(created.decision, AuditDecision::Allow);

    let refused = ibkr_orders::build_order_audit_event(
        AuditEventType::OrderPreviewRefused,
        AuditResultStatus::Refused,
    );
    assert_eq!(refused.tool_name.as_deref(), Some("ibkr_order_preview"));
    assert_eq!(refused.decision, AuditDecision::Refuse);
}
