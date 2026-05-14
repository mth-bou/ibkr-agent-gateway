//! Order preview audit helpers.

use ibkr_audit::{AuditDecision, AuditEvent, AuditEventType, AuditResultStatus};
use ibkr_domain::{AuditEventId, LocalUserId, RequestId, SessionId};
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// Builds a preview-phase audit event.
#[must_use]
pub fn build_order_audit_event(
    event_type: AuditEventType,
    result_status: AuditResultStatus,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type,
        timestamp: OffsetDateTime::now_utc(),
        user_id: LocalUserId::from_static("local-user"),
        session_id: SessionId::new(),
        request_id: RequestId::new(),
        account_id_hash: None,
        tool_name: Some("ibkr_order_preview".to_string()),
        scopes: vec!["ibkr:orders:preview".to_string()],
        decision: if matches!(result_status, AuditResultStatus::Refused) {
            AuditDecision::Refuse
        } else {
            AuditDecision::Allow
        },
        result_status,
        error_code: None,
        input_hash: None,
        output_hash: None,
        redactions: Vec::new(),
        metadata: BTreeMap::new(),
    }
}

/// Builds a paper-order audit event.
#[must_use]
pub fn build_paper_order_audit_event(
    tool_name: &str,
    scope: &str,
    event_type: AuditEventType,
    result_status: AuditResultStatus,
) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type,
        timestamp: OffsetDateTime::now_utc(),
        user_id: LocalUserId::from_static("local-user"),
        session_id: SessionId::new(),
        request_id: RequestId::new(),
        account_id_hash: None,
        tool_name: Some(tool_name.to_string()),
        scopes: vec![scope.to_string()],
        decision: if matches!(result_status, AuditResultStatus::Refused) {
            AuditDecision::Refuse
        } else {
            AuditDecision::Allow
        },
        result_status,
        error_code: None,
        input_hash: None,
        output_hash: None,
        redactions: Vec::new(),
        metadata: BTreeMap::new(),
    }
}
