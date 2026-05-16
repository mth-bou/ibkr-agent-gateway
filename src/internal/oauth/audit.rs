//! OAuth audit helpers.

use crate::internal::audit::{
    AuditDecision, AuditEvent, AuditEventType, AuditResultStatus, RedactionRecord,
};
use crate::internal::domain::{
    AccountIdHash, AuditEventId, ErrorCode, LocalUserId, RequestId, SessionId,
};
use serde_json::json;
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// Builds a redacted remote auth audit event.
#[must_use]
pub fn build_remote_auth_audit_event(
    subject: &str,
    issuer: &str,
    scopes: Vec<String>,
    result_status: AuditResultStatus,
    error_code: Option<ErrorCode>,
    token_id_hash: Option<AccountIdHash>,
) -> AuditEvent {
    let mut metadata = BTreeMap::new();
    metadata.insert("subject".to_string(), json!(subject));
    metadata.insert("issuer".to_string(), json!(issuer));
    metadata.insert(
        "token_id_hash".to_string(),
        json!(token_id_hash.as_ref().map(AccountIdHash::as_str)),
    );

    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: match result_status {
            AuditResultStatus::Authenticated => AuditEventType::RemoteAuthSucceeded,
            _ => AuditEventType::RemoteAuthDenied,
        },
        timestamp: OffsetDateTime::now_utc(),
        user_id: LocalUserId::new(format!("remote:{subject}"))
            .unwrap_or_else(|| LocalUserId::from_static("remote-user")),
        session_id: SessionId::new(),
        request_id: RequestId::new(),
        account_id_hash: None,
        tool_name: None,
        scopes,
        decision: match result_status {
            AuditResultStatus::Authenticated => AuditDecision::Allow,
            AuditResultStatus::DeniedScope | AuditResultStatus::DeniedAuth => AuditDecision::Deny,
            _ => AuditDecision::Refuse,
        },
        result_status,
        error_code,
        input_hash: None,
        output_hash: None,
        redactions: vec![RedactionRecord {
            field_path: "headers.authorization".to_string(),
            reason: "bearer_token".to_string(),
        }],
        metadata,
    }
}
