//! Append-only audit models, redaction, HMAC identifiers, and persistence.

pub mod account_hash;
pub mod event;
pub mod export;
pub mod query;
pub mod recorder;
pub mod redaction;
pub mod replay;
pub mod sqlite;

pub use account_hash::{AuditHmacKey, MIN_AUDIT_HMAC_KEY_BYTES};
pub use event::{AuditDecision, AuditEvent, AuditEventType, AuditResultStatus, RedactionRecord};
pub use export::{AuditExport, AuditExportFormat, AuditExportRange, export_audit_tail_jsonl};
pub use query::{AuditTail, AuditTailRecord, AuditTailRequest};
pub use recorder::AuditRecorder;
pub use redaction::{hmac_sha256_hex, is_sensitive_field_name, scrub_audit_metadata, sha256_hex};
pub use replay::{ReplayCase, ReplayOutcome, SecretScanExpectation, replay_case};
pub use sqlite::{OrderPreviewRecord, SqliteAuditWriter};

#[cfg(test)]
mod tests {
    use super::{hmac_sha256_hex, is_sensitive_field_name, sha256_hex};

    #[test]
    fn hmac_hash_is_deterministic() -> Result<(), hmac::digest::InvalidLength> {
        let first = hmac_sha256_hex(b"secret", b"U1234567")?;
        let second = hmac_sha256_hex(b"secret", b"U1234567")?;
        assert_eq!(first, second);
        assert_ne!(first, sha256_hex(b"U1234567"));
        Ok(())
    }

    #[test]
    fn detects_sensitive_field_names() {
        assert!(is_sensitive_field_name("Authorization"));
        assert!(is_sensitive_field_name("session_cookie"));
        assert!(!is_sensitive_field_name("account_mode"));
    }
}
