//! CLI audit event helpers.

use ibkr_audit::{AuditDecision, AuditEvent, AuditEventType, AuditResultStatus};
use ibkr_domain::{AuditEventId, LocalUserId, RequestId, SessionId};
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// Builds a minimal CLI audit event for US1 operations.
#[must_use]
pub fn build_cli_audit_event(
    tool_name: impl Into<String>,
    scope: impl Into<String>,
    event_type: AuditEventType,
    result_status: AuditResultStatus,
) -> AuditEvent {
    let user_id = LocalUserId::from_static("local-user");
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type,
        timestamp: OffsetDateTime::now_utc(),
        user_id,
        session_id: SessionId::new(),
        request_id: RequestId::new(),
        account_id_hash: None,
        tool_name: Some(tool_name.into()),
        scopes: vec![scope.into()],
        decision: AuditDecision::Allow,
        result_status,
        error_code: None,
        input_hash: None,
        output_hash: None,
        redactions: Vec::new(),
        metadata: BTreeMap::new(),
    }
}
