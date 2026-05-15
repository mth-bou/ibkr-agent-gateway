//! Audit review commands.

use crate::output::print_output;
use ibkr_audit::{AuditTailRequest, SqliteAuditWriter};
use ibkr_domain::GatewayError;

/// Reads recent audit events from SQLite.
pub async fn tail(database_url: &str, limit: u32, json: bool) -> Result<(), GatewayError> {
    let writer = SqliteAuditWriter::connect(database_url).await?;
    let output = writer.tail(AuditTailRequest::new(limit)).await?;
    print_output(json, "audit tail loaded", &output)
}

/// Exports recent audit events as JSONL.
pub async fn export(database_url: &str, limit: u32, json: bool) -> Result<(), GatewayError> {
    let writer = SqliteAuditWriter::connect(database_url).await?;
    let output = writer.export_jsonl(AuditTailRequest::new(limit)).await?;
    if json {
        print_output(true, "audit export loaded", &output)
    } else {
        print!("{}", output.payload_jsonl);
        Ok(())
    }
}
