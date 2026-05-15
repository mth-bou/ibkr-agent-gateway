//! Sidecar relay audit helpers.

use crate::ForwardedBrokerRequest;
use ibkr_audit::{AuditDecision, AuditEvent, AuditEventType, AuditResultStatus};
use ibkr_domain::{AuditEventId, ErrorCode, LocalUserId, SessionId};
use serde_json::json;
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// Builds a sidecar relay audit event without storing forwarded payloads or session material.
#[must_use]
pub fn build_sidecar_relay_audit_event(
    request: &ForwardedBrokerRequest,
    remote_instance_id: &str,
    status: AuditResultStatus,
    error_code: Option<ErrorCode>,
) -> AuditEvent {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "remote_instance_id".to_string(),
        json!(remote_instance_id.to_string()),
    );
    metadata.insert("payload_hash".to_string(), json!(request.payload_hash));

    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: match status {
            AuditResultStatus::Completed => AuditEventType::SidecarRelayForwarded,
            _ => AuditEventType::SidecarRelayFailed,
        },
        timestamp: OffsetDateTime::now_utc(),
        user_id: LocalUserId::from_static("sidecar-relay"),
        session_id: SessionId::new(),
        request_id: request.request_id.clone(),
        account_id_hash: None,
        tool_name: Some(request.tool_name.clone()),
        scopes: vec![request.scope.clone()],
        decision: match status {
            AuditResultStatus::Completed => AuditDecision::Allow,
            AuditResultStatus::DeniedScope | AuditResultStatus::DeniedAuth => AuditDecision::Deny,
            _ => AuditDecision::Refuse,
        },
        result_status: status,
        error_code,
        input_hash: Some(request.payload_hash.clone()),
        output_hash: None,
        redactions: Vec::new(),
        metadata,
    }
}
