//! Secret-safe operational metrics.

use ibkr_audit::is_sensitive_field_name;
use ibkr_domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// One metric event safe for external observability systems.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MetricEvent {
    /// Metric name.
    pub metric_name: String,
    /// Low-cardinality labels.
    pub labels: BTreeMap<String, String>,
    /// Metric value.
    pub value: f64,
    /// Event timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub timestamp: OffsetDateTime,
}

impl MetricEvent {
    /// Creates a metric event after validating labels.
    pub fn new(
        metric_name: impl Into<String>,
        labels: BTreeMap<String, String>,
        value: f64,
    ) -> Result<Self, GatewayError> {
        let event = Self {
            metric_name: metric_name.into(),
            labels,
            value,
            timestamp: OffsetDateTime::now_utc(),
        };
        validate_metric_event(&event)?;
        Ok(event)
    }
}

/// Validates that a metric event does not expose sensitive labels.
pub fn validate_metric_event(event: &MetricEvent) -> Result<(), GatewayError> {
    for (key, value) in &event.labels {
        if is_sensitive_label_key(key) || is_sensitive_label_value(value) {
            return Err(GatewayError::new(
                ErrorCode::OutputUnsafe,
                "Metric label contains sensitive material",
                false,
                Some("Use low-cardinality non-sensitive metric labels".to_string()),
            ));
        }
    }

    Ok(())
}

fn is_sensitive_label_key(key: &str) -> bool {
    is_sensitive_field_name(key)
        || key.eq_ignore_ascii_case("account_id")
        || key.eq_ignore_ascii_case("broker_order_id")
        || key.eq_ignore_ascii_case("session_id")
}

fn is_sensitive_label_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("bearer ")
        || lowered.contains("cookie=")
        || lowered.contains("client_secret")
        || lowered.contains("refresh_token")
        || lowered.contains("password=")
        || lowered.contains("/home/")
        || lowered.contains("\\users\\")
        || looks_like_account_id(value)
}

fn looks_like_account_id(value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    (upper.starts_with('U') || upper.starts_with("DU"))
        && upper
            .chars()
            .skip_while(|character| character.is_ascii_alphabetic())
            .all(|character| character.is_ascii_digit())
        && upper.chars().any(|character| character.is_ascii_digit())
}
