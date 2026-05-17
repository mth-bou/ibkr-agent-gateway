//! Audit event model and required event types.

use crate::internal::domain::{
    AccountIdHash, AuditEventId, ErrorCode, LocalUserId, RequestId, SessionId,
};
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
    /// A tool call was refused by policy.
    ToolRefused,
    /// Broker session state changed.
    BackendSessionChanged,
    /// Broker session was checked.
    BackendSessionChecked,
    /// Order intent was received.
    OrderIntentReceived,
    /// Order risk checks completed.
    OrderRiskChecked,
    /// Order preview was created.
    OrderPreviewCreated,
    /// Order preview was refused.
    OrderPreviewRefused,
    /// Paper approval was created or updated.
    PaperApprovalRecorded,
    /// Paper order submit was accepted or refused.
    PaperOrderSubmitted,
    /// Paper order cancel was accepted or refused.
    PaperOrderCancelled,
    /// Paper order lifecycle changed.
    PaperOrderLifecycleChanged,
    /// Remote OAuth/OIDC authentication succeeded.
    RemoteAuthSucceeded,
    /// Remote OAuth/OIDC authentication was denied.
    RemoteAuthDenied,
    /// Sidecar relay forwarded a broker request.
    SidecarRelayForwarded,
    /// Sidecar relay refused or failed a broker request.
    SidecarRelayFailed,
}

impl AuditEventType {
    /// Stable serialized audit event type name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ToolCalled => "tool_called",
            Self::ToolDeniedScope => "tool_denied_scope",
            Self::ToolCompleted => "tool_completed",
            Self::ToolFailed => "tool_failed",
            Self::ToolRefused => "tool_refused",
            Self::BackendSessionChanged => "backend_session_changed",
            Self::BackendSessionChecked => "backend_session_checked",
            Self::OrderIntentReceived => "order_intent_received",
            Self::OrderRiskChecked => "order_risk_checked",
            Self::OrderPreviewCreated => "order_preview_created",
            Self::OrderPreviewRefused => "order_preview_refused",
            Self::PaperApprovalRecorded => "paper_approval_recorded",
            Self::PaperOrderSubmitted => "paper_order_submitted",
            Self::PaperOrderCancelled => "paper_order_cancelled",
            Self::PaperOrderLifecycleChanged => "paper_order_lifecycle_changed",
            Self::RemoteAuthSucceeded => "remote_auth_succeeded",
            Self::RemoteAuthDenied => "remote_auth_denied",
            Self::SidecarRelayForwarded => "sidecar_relay_forwarded",
            Self::SidecarRelayFailed => "sidecar_relay_failed",
        }
    }
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
    /// Remote auth succeeded.
    Authenticated,
    /// Remote auth failed before tool execution.
    DeniedAuth,
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
