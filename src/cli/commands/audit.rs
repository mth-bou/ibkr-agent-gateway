//! Audit review commands.

use crate::cli::output::print_output;
use crate::internal::audit::{AuditHmacKey, AuditTailRequest, SqliteAuditWriter};
use crate::internal::domain::GatewayError;
use std::sync::Arc;

/// Reads recent audit events from SQLite.
pub async fn tail(database_url: &str, limit: u32, json: bool) -> Result<(), GatewayError> {
    let writer = SqliteAuditWriter::connect(database_url, cli_audit_key()?).await?;
    let output = writer.tail(AuditTailRequest::new(limit)).await?;
    print_output(json, "audit tail loaded", &output)
}

/// Exports recent audit events as JSONL.
pub async fn export(database_url: &str, limit: u32, json: bool) -> Result<(), GatewayError> {
    let writer = SqliteAuditWriter::connect(database_url, cli_audit_key()?).await?;
    let output = writer.export_jsonl(AuditTailRequest::new(limit)).await?;
    if json {
        print_output(true, "audit export loaded", &output)
    } else {
        print!("{}", output.payload_jsonl);
        Ok(())
    }
}

/// CLI audit commands only read existing rows; verification requires the
/// original write-time key, which is operator-supplied configuration not yet
/// wired into the CLI. Use an ephemeral key so the writer can still load and
/// surface stored events even though it cannot itself verify chain integrity.
fn cli_audit_key() -> Result<Arc<AuditHmacKey>, GatewayError> {
    Ok(Arc::new(AuditHmacKey::ephemeral()?))
}
