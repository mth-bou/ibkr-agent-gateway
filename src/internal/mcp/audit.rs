//! MCP audit helpers.

use ibkr_audit::{AuditDecision, AuditEvent, AuditEventType, AuditResultStatus};
use ibkr_domain::{AuditEventId, LocalUserId, RequestId, SessionId};
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// Builds a minimal MCP tool audit event.
#[must_use]
pub fn build_mcp_tool_event(tool_name: &str, scope: &str, status: AuditResultStatus) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: match status {
            AuditResultStatus::DeniedScope => AuditEventType::ToolDeniedScope,
            AuditResultStatus::Failed => AuditEventType::ToolFailed,
            AuditResultStatus::Completed => AuditEventType::ToolCompleted,
            AuditResultStatus::Refused => AuditEventType::ToolRefused,
            _ => AuditEventType::ToolCalled,
        },
        timestamp: OffsetDateTime::now_utc(),
        user_id: LocalUserId::from_static("local-user"),
        session_id: SessionId::new(),
        request_id: RequestId::new(),
        account_id_hash: None,
        tool_name: Some(tool_name.to_string()),
        scopes: vec![scope.to_string()],
        decision: match status {
            AuditResultStatus::DeniedScope => AuditDecision::Deny,
            AuditResultStatus::Refused => AuditDecision::Refuse,
            _ => AuditDecision::Allow,
        },
        result_status: status,
        error_code: None,
        input_hash: None,
        output_hash: None,
        redactions: Vec::new(),
        metadata: BTreeMap::new(),
    }
}
