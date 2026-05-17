//! Paper order idempotency models.

use crate::internal::domain::{ErrorCode, GatewayError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime};

const DEFAULT_MAX_RECORDS: usize = 10_000;
const DEFAULT_TTL: Duration = Duration::hours(24);

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

/// Result of an idempotency lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdempotencyDecision {
    /// First time this key is seen.
    New,
    /// Same key and same request hash were already recorded.
    Replayed,
}

/// In-memory idempotency store for local paper workflow tests and adapters.
#[derive(Clone, Debug)]
pub struct IdempotencyStore {
    records: BTreeMap<String, IdempotencyRecord>,
    max_records: usize,
    ttl: Duration,
}

impl IdempotencyStore {
    /// Creates a bounded in-memory idempotency store.
    #[must_use]
    pub const fn bounded(max_records: usize, ttl: Duration) -> Self {
        Self {
            records: BTreeMap::new(),
            max_records,
            ttl,
        }
    }

    /// Returns the number of currently retained idempotency records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true when no idempotency records are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Records or replays an idempotency key.
    pub fn record_or_replay(
        &mut self,
        key: IdempotencyKey,
        request_hash: impl Into<String>,
    ) -> Result<IdempotencyDecision, GatewayError> {
        let now = OffsetDateTime::now_utc();
        self.purge_expired(now);

        let request_hash = request_hash.into();
        if let Some(existing) = self.records.get(key.as_str()) {
            if existing.request_hash == request_hash {
                return Ok(IdempotencyDecision::Replayed);
            }
            return Err(GatewayError::new(
                ErrorCode::PaperIdempotencyConflict,
                "Idempotency key conflicts with a previous request",
                false,
                Some("Use a new idempotency key for a different request".to_string()),
            ));
        }

        if self.max_records == 0 {
            return Err(GatewayError::new(
                ErrorCode::PaperIdempotencyConflict,
                "Idempotency store capacity is zero",
                true,
                Some("Configure a positive idempotency store capacity".to_string()),
            ));
        }
        if self.records.len() >= self.max_records {
            self.evict_oldest();
        }

        self.records.insert(
            key.as_str().to_string(),
            IdempotencyRecord {
                key,
                request_hash,
                result_hash: None,
                created_at: now,
            },
        );
        Ok(IdempotencyDecision::New)
    }

    fn purge_expired(&mut self, now: OffsetDateTime) {
        let ttl = self.ttl;
        self.records
            .retain(|_, record| now - record.created_at <= ttl);
    }

    fn evict_oldest(&mut self) {
        let Some(oldest_key) = self
            .records
            .iter()
            .min_by_key(|(_, record)| record.created_at)
            .map(|(key, _)| key.clone())
        else {
            return;
        };
        self.records.remove(&oldest_key);
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        Self::bounded(DEFAULT_MAX_RECORDS, DEFAULT_TTL)
    }
}

/// Builds a stable SHA-256 request hash from canonical JSON and a flow namespace.
pub fn stable_request_hash<T: Serialize>(
    namespace: &str,
    request: &T,
) -> Result<String, GatewayError> {
    let request_json = serde_json::to_vec(request).map_err(|_| {
        GatewayError::new(
            ErrorCode::PaperIdempotencyConflict,
            "Unable to build idempotency request hash",
            false,
            Some("Submit a serializable order request".to_string()),
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    hasher.update(b":");
    hasher.update(&request_json);
    Ok(bytes_to_lower_hex(&hasher.finalize()))
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
