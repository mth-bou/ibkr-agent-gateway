//! Paper order idempotency models.

use ibkr_domain::{ErrorCode, GatewayError};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Idempotency key for paper submit/cancel requests.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Creates a non-empty idempotency key.
    pub fn new(value: impl Into<String>) -> Result<Self, GatewayError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(GatewayError::new(
                ErrorCode::PaperIdempotencyConflict,
                "Idempotency key is required",
                false,
                Some("Provide a stable idempotency key".to_string()),
            ))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the raw key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stored idempotency record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    /// Idempotency key.
    pub key: IdempotencyKey,
    /// Canonical request hash.
    pub request_hash: String,
    /// Stored result hash when completed.
    pub result_hash: Option<String>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}
