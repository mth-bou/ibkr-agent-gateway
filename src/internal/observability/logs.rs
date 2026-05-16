//! Secret-safe structured log helpers.

use ibkr_audit::is_sensitive_field_name;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Structured log event with redacted fields.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StructuredLogEvent {
    /// Event name.
    pub event_name: String,
    /// Sanitized fields.
    pub fields: BTreeMap<String, String>,
}

impl StructuredLogEvent {
    /// Creates a log event after sanitizing fields.
    #[must_use]
    pub fn new(event_name: impl Into<String>, fields: BTreeMap<String, String>) -> Self {
        Self {
            event_name: event_name.into(),
            fields: sanitize_log_fields(fields),
        }
    }
}

/// Redacts sensitive structured log fields.
#[must_use]
pub fn sanitize_log_fields(fields: BTreeMap<String, String>) -> BTreeMap<String, String> {
    fields
        .into_iter()
        .map(|(key, value)| {
            if is_sensitive_field_name(&key) || is_secret_like_value(&value) {
                (key, "[REDACTED]".to_string())
            } else {
                (key, value)
            }
        })
        .collect()
}

fn is_secret_like_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("bearer ")
        || lowered.contains("cookie=")
        || lowered.contains("client_secret")
        || lowered.contains("refresh_token")
        || lowered.contains("password=")
        || lowered.contains("/home/")
        || lowered.contains("\\users\\")
}
