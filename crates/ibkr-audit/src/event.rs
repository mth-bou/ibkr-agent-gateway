//! Audit event model and required event types.

use ibkr_domain::{AccountIdHash, AuditEventId, ErrorCode, LocalUserId, RequestId, SessionId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// Required audit event types for the read-only MVP.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// A tool call was received.
    ToolCalled,
    /// A tool call was denied because scope was missing.
    ToolDeniedScope,
    /// A tool call completed.
    ToolCompleted,
    /// A tool call failed.
    ToolFailed,
    /// Broker session state changed.
    BackendSessionChanged,
    /// Broker session was checked.
    BackendSessionChecked,
}

/// Authorization decision captured in audit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditDecision {
    /// Operation was allowed.
    Allow,
    /// Operation was denied by auth/scope.
    Deny,
    /// Operation was refused by validation or policy.
    Refuse,
}

/// Terminal result status captured in audit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditResultStatus {
    /// Call was recorded.
    Called,
    /// Call completed.
    Completed,
    /// Call failed.
    Failed,
    /// Call was refused.
    Refused,
    /// Call was denied for missing scope.
    DeniedScope,
}

/// Redaction metadata for one sensitive field.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RedactionRecord {
    /// Redacted field path.
    pub field_path: String,
    /// Redaction reason.
    pub reason: String,
}

/// Append-only audit event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event id.
    pub event_id: AuditEventId,
    /// Event type.
    pub event_type: AuditEventType,
    /// Event timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    /// Local user id.
    pub user_id: LocalUserId,
    /// Session id.
    pub session_id: SessionId,
    /// Request id.
    pub request_id: RequestId,
    /// HMAC account hash when applicable.
    pub account_id_hash: Option<AccountIdHash>,
    /// Tool name when applicable.
    pub tool_name: Option<String>,
    /// Scopes involved in the decision.
    pub scopes: Vec<String>,
    /// Allow, deny, or refuse.
    pub decision: AuditDecision,
    /// Result status.
    pub result_status: AuditResultStatus,
    /// Optional stable error code.
    pub error_code: Option<ErrorCode>,
    /// Canonical input hash.
    pub input_hash: Option<String>,
    /// Canonical output hash.
    pub output_hash: Option<String>,
    /// Redaction metadata.
    pub redactions: Vec<RedactionRecord>,
    /// Non-sensitive metadata.
    pub metadata: BTreeMap<String, Value>,
}
