//! Redaction and HMAC-SHA256 helpers.

use super::event::RedactionRecord;
use crate::internal::encoding::bytes_to_lower_hex;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

/// Sensitive field names that must not be stored raw.
pub const SENSITIVE_FIELD_NAMES: &[&str] = &[
    "token",
    "access_token",
    "refresh_token",
    "cookie",
    "set-cookie",
    "authorization",
    "password",
    "credential",
    "secret",
    "header",
    "path",
];

/// Computes a lowercase hex SHA-256 hash for canonical payloads.
#[must_use]
pub fn sha256_hex(value: &[u8]) -> String {
    crate::internal::encoding::sha256_hex(value)
}

/// Computes a lowercase hex HMAC-SHA256 identifier.
pub fn hmac_sha256_hex(secret: &[u8], value: &[u8]) -> Result<String, hmac::digest::InvalidLength> {
    let mut mac = HmacSha256::new_from_slice(secret)?;
    mac.update(value);
    let result = mac.finalize().into_bytes();
    Ok(bytes_to_lower_hex(&result))
}

/// Returns true when a field name is sensitive.
#[must_use]
pub fn is_sensitive_field_name(name: &str) -> bool {
    SENSITIVE_FIELD_NAMES.iter().any(|sensitive| {
        let needle = sensitive.as_bytes();
        name.len() >= needle.len()
            && name
                .as_bytes()
                .windows(needle.len())
                .any(|window| window.eq_ignore_ascii_case(needle))
    })
}

/// Scrubs an audit metadata map by name and returns the scrubbed map plus a
/// matching redaction trail. Sensitive keys keep their position in the map but
/// their value is replaced by `"[REDACTED]"` so downstream consumers retain
/// shape stability while never seeing the raw value.
#[must_use]
pub fn scrub_audit_metadata(
    metadata: BTreeMap<String, Value>,
) -> (BTreeMap<String, Value>, Vec<RedactionRecord>) {
    let mut redactions = Vec::new();
    let scrubbed = metadata
        .into_iter()
        .map(|(key, value)| {
            if is_sensitive_field_name(&key) {
                redactions.push(RedactionRecord {
                    field_path: format!("metadata.{key}"),
                    reason: "sensitive field name".to_string(),
                });
                (key, Value::String("[REDACTED]".to_string()))
            } else {
                (key, value)
            }
        })
        .collect();
    (scrubbed, redactions)
}

#[cfg(test)]
mod scrub_tests {
    use super::{is_sensitive_field_name, scrub_audit_metadata};
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    #[test]
    fn redacts_sensitive_metadata_keys() {
        let mut metadata = BTreeMap::new();
        metadata.insert("authorization".to_string(), json!("Bearer abc.def"));
        metadata.insert("symbol".to_string(), json!("AAPL"));

        let (scrubbed, redactions) = scrub_audit_metadata(metadata);

        assert_eq!(
            scrubbed.get("authorization"),
            Some(&Value::String("[REDACTED]".to_string()))
        );
        assert_eq!(scrubbed.get("symbol"), Some(&json!("AAPL")));
        assert_eq!(redactions.len(), 1);
        assert_eq!(redactions[0].field_path, "metadata.authorization");
    }

    #[test]
    fn leaves_non_sensitive_metadata_unchanged() {
        let mut metadata = BTreeMap::new();
        metadata.insert("symbol".to_string(), json!("AAPL"));
        metadata.insert("quantity".to_string(), json!(100));

        let (scrubbed, redactions) = scrub_audit_metadata(metadata.clone());

        assert_eq!(scrubbed, metadata);
        assert!(redactions.is_empty());
    }

    #[test]
    fn redacts_value_regardless_of_payload_shape() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "session_cookie".to_string(),
            json!({"nested": {"deep": "secret"}}),
        );

        let (scrubbed, redactions) = scrub_audit_metadata(metadata);

        assert_eq!(
            scrubbed.get("session_cookie"),
            Some(&Value::String("[REDACTED]".to_string()))
        );
        assert_eq!(redactions.len(), 1);
    }

    #[test]
    fn detects_sensitive_names_case_insensitively_without_exact_case() {
        assert!(is_sensitive_field_name("X-AuthORIZATION-Header"));
        assert!(is_sensitive_field_name("session_cookie"));
        assert!(!is_sensitive_field_name("symbol"));
    }
}
