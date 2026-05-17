//! HMAC-SHA256 key material used to redact account identifiers in audit records.
//!
//! The key is constructed once at startup (either supplied via configuration or
//! generated as an ephemeral per-process secret) and shared with every code path
//! that derives an [`AccountIdHash`]. Centralising construction here prevents
//! accidental reintroduction of placeholder hashes such as the one removed in the
//! 2026-05-17 review (C-1).

use super::redaction::hmac_sha256_hex;
use crate::internal::domain::{AccountIdHash, ErrorCode, GatewayError};
use ring::rand::{SecureRandom, SystemRandom};

/// Minimum entropy size for an audit HMAC key (32 bytes = SHA-256 block size).
pub const MIN_AUDIT_HMAC_KEY_BYTES: usize = 32;

/// Key material used to HMAC account identifiers before audit storage.
///
/// Construction enforces a minimum length of [`MIN_AUDIT_HMAC_KEY_BYTES`]. The
/// type intentionally suppresses key material from [`std::fmt::Debug`] and
/// performs constant-time equality comparisons.
#[derive(Clone)]
pub struct AuditHmacKey(Vec<u8>);

impl AuditHmacKey {
    /// Constructs a key from caller-supplied bytes.
    ///
    /// Returns a configuration error when the byte slice is shorter than
    /// [`MIN_AUDIT_HMAC_KEY_BYTES`].
    pub fn new(bytes: impl Into<Vec<u8>>) -> Result<Self, GatewayError> {
        let bytes = bytes.into();
        if bytes.len() < MIN_AUDIT_HMAC_KEY_BYTES {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                format!("Audit HMAC key must be at least {MIN_AUDIT_HMAC_KEY_BYTES} bytes"),
                false,
                Some("Provide a 32-byte or longer random secret via configuration".to_string()),
            ));
        }
        Ok(Self(bytes))
    }

    /// Generates a fresh ephemeral key using the operating system RNG.
    ///
    /// Suitable for processes where audit hashes only need to be stable for the
    /// lifetime of the gateway (developer machines, CI, ephemeral containers).
    pub fn ephemeral() -> Result<Self, GatewayError> {
        let mut bytes = vec![0_u8; MIN_AUDIT_HMAC_KEY_BYTES];
        SystemRandom::new().fill(&mut bytes).map_err(|_| {
            GatewayError::new(
                ErrorCode::AuditWriteFailed,
                "Failed to generate ephemeral audit HMAC key",
                true,
                Some("Retry gateway startup".to_string()),
            )
        })?;
        Ok(Self(bytes))
    }

    /// Computes the HMAC-SHA256 of `account_id` under this key.
    pub fn compute_account_id_hash(&self, account_id: &str) -> Result<AccountIdHash, GatewayError> {
        let hex = hmac_sha256_hex(&self.0, account_id.as_bytes()).map_err(|_| {
            GatewayError::new(
                ErrorCode::AuditWriteFailed,
                "Failed to compute audit HMAC over account identifier",
                true,
                Some("Retry the operation".to_string()),
            )
        })?;
        Ok(AccountIdHash::from_hash(hex))
    }

    /// Returns the raw key bytes. Intentionally `pub(crate)`: only the audit
    /// module is allowed to introduce new HMAC use sites.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for AuditHmacKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuditHmacKey")
            .field("len", &self.0.len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for AuditHmacKey {
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq(&self.0, &other.0)
    }
}

impl Eq for AuditHmacKey {}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (left, right) in a.iter().zip(b.iter()) {
        diff |= left ^ right;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::{AuditHmacKey, MIN_AUDIT_HMAC_KEY_BYTES};
    use crate::internal::domain::ErrorCode;

    fn sample_key() -> AuditHmacKey {
        let Ok(key) = AuditHmacKey::new(vec![0x42_u8; MIN_AUDIT_HMAC_KEY_BYTES]) else {
            unreachable!("32-byte literal key should construct");
        };
        key
    }

    #[test]
    fn rejects_keys_shorter_than_minimum() {
        let Err(error) = AuditHmacKey::new(vec![0_u8; MIN_AUDIT_HMAC_KEY_BYTES - 1]) else {
            unreachable!("31-byte key should be rejected");
        };
        assert_eq!(error.code, ErrorCode::ConfigInvalid);
    }

    #[test]
    fn accepts_minimum_length_keys() {
        let key = AuditHmacKey::new(vec![0_u8; MIN_AUDIT_HMAC_KEY_BYTES]);
        assert!(key.is_ok());
    }

    #[test]
    fn hash_is_deterministic_for_same_key_and_input() {
        let key = sample_key();
        let Ok(first) = key.compute_account_id_hash("DU1234567") else {
            unreachable!("hash must succeed");
        };
        let Ok(second) = key.compute_account_id_hash("DU1234567") else {
            unreachable!("hash must succeed");
        };
        assert_eq!(first.as_str(), second.as_str());
    }

    #[test]
    fn different_keys_produce_different_hashes() {
        let Ok(key_a) = AuditHmacKey::new(vec![0xAA_u8; MIN_AUDIT_HMAC_KEY_BYTES]) else {
            unreachable!("key A should construct");
        };
        let Ok(key_b) = AuditHmacKey::new(vec![0xBB_u8; MIN_AUDIT_HMAC_KEY_BYTES]) else {
            unreachable!("key B should construct");
        };
        let Ok(hash_a) = key_a.compute_account_id_hash("DU1234567") else {
            unreachable!("hash A must succeed");
        };
        let Ok(hash_b) = key_b.compute_account_id_hash("DU1234567") else {
            unreachable!("hash B must succeed");
        };
        assert_ne!(hash_a.as_str(), hash_b.as_str());
    }

    #[test]
    fn hash_does_not_leak_raw_account_id() {
        let key = sample_key();
        let Ok(hash) = key.compute_account_id_hash("DU1234567") else {
            unreachable!("hash must succeed");
        };
        assert!(!hash.as_str().contains("DU1234567"));
        assert!(!hash.as_str().contains("fixture-hmac"));
        assert_eq!(hash.as_str().len(), 64);
        assert!(hash.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn ephemeral_keys_are_distinct() {
        let Ok(first) = AuditHmacKey::ephemeral() else {
            unreachable!("ephemeral key must generate");
        };
        let Ok(second) = AuditHmacKey::ephemeral() else {
            unreachable!("ephemeral key must generate");
        };
        assert_ne!(first, second);
    }

    #[test]
    fn debug_does_not_expose_key_material() {
        let key = sample_key();
        let rendered = format!("{key:?}");
        assert!(!rendered.contains("66")); // 0x42 = 66 decimal
        assert!(!rendered.contains("42"));
    }
}
