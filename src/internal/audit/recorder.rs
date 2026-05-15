//! Shared record-only audit recorder.

use super::event::AuditEvent;
use super::sqlite::SqliteAuditWriter;
use ibkr_domain::GatewayError;

/// Shared audit recorder used by CLI and MCP operations.
#[derive(Clone)]
pub struct AuditRecorder {
    writer: SqliteAuditWriter,
}

impl AuditRecorder {
    /// Creates a recorder from a SQLite writer.
    #[must_use]
    pub const fn new(writer: SqliteAuditWriter) -> Self {
        Self { writer }
    }

    /// Records one audit event.
    pub async fn record(&self, event: &AuditEvent) -> Result<(), GatewayError> {
        self.writer.append(event).await
    }
}
