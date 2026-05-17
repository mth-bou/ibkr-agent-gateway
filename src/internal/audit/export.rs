//! Redacted audit export helpers.

use super::{
    query::{AuditTail, AuditTailRecord},
    redaction::sha256_hex,
};
use crate::internal::domain::{AuditEventId, ErrorCode, GatewayError};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Audit export format.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditExportFormat {
    /// Newline-delimited JSON records.
    Jsonl,
}

/// Exported audit range summary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditExportRange {
    /// Requested tail limit.
    pub requested_limit: u32,
    /// Exported record count.
    pub exported_count: usize,
}

/// Redacted audit export.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditExport {
    /// Export id.
    pub export_id: AuditEventId,
    /// Export format.
    pub format: AuditExportFormat,
    /// Export range.
    pub range: AuditExportRange,
    /// Redaction policy summary.
    pub redaction_policy: String,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// SHA-256 hash of the JSONL payload.
    pub file_hash: String,
    /// JSONL payload.
    pub payload_jsonl: String,
}

/// Exports audit tail records as redacted JSONL.
pub fn export_audit_tail_jsonl(
    tail: &AuditTail,
    requested_limit: u32,
) -> Result<AuditExport, GatewayError> {
    let payload_jsonl = render_jsonl(&tail.events)?;

    Ok(AuditExport {
        export_id: AuditEventId::new(),
        format: AuditExportFormat::Jsonl,
        range: AuditExportRange {
            requested_limit,
            exported_count: tail.events.len(),
        },
        redaction_policy: "redacted_audit_events_only".to_string(),
        created_at: OffsetDateTime::now_utc(),
        file_hash: sha256_hex(payload_jsonl.as_bytes()),
        payload_jsonl,
    })
}

fn render_jsonl(records: &[AuditTailRecord]) -> Result<String, GatewayError> {
    let mut output = String::with_capacity(records.len().saturating_mul(256));
    for record in records {
        let line = serde_json::to_string(record).map_err(|_| {
            GatewayError::new(
                ErrorCode::OutputUnsafe,
                "Failed to serialize audit export record",
                false,
                Some("Inspect audit export serialization".to_string()),
            )
        })?;
        assert_secret_safe_line(&line)?;
        output.push_str(&line);
        output.push('\n');
    }
    Ok(output)
}

fn assert_secret_safe_line(line: &str) -> Result<(), GatewayError> {
    let lowered = line.to_ascii_lowercase();
    for marker in [
        "bearer ",
        "client_secret",
        "cookie=",
        "refresh_token",
        "password=",
        "/home/",
        "\\users\\",
    ] {
        if lowered.contains(marker) {
            return Err(GatewayError::new(
                ErrorCode::OutputUnsafe,
                "Audit export contains secret-like material",
                false,
                Some("Redact audit payloads before export".to_string()),
            ));
        }
    }
    Ok(())
}
