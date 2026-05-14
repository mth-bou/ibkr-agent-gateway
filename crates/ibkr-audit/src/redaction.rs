//! Redaction and HMAC-SHA256 helpers.

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

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
    let digest = Sha256::digest(value);
    bytes_to_lower_hex(&digest)
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
    let lowered = name.to_ascii_lowercase();
    SENSITIVE_FIELD_NAMES
        .iter()
        .any(|sensitive| lowered.contains(sensitive))
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
