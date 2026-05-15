//! SQLite append-only audit persistence.

use super::query::{AuditTail, AuditTailRecord, AuditTailRequest};
use super::{
    event::AuditEvent,
    export::{AuditExport, export_audit_tail_jsonl},
};
use ibkr_domain::{ErrorCode, GatewayError};
use sqlx_core::{Error as SqlxError, query::query, row::Row};
use sqlx_sqlite::{SqlitePool, SqlitePoolOptions};

/// SQLite-backed audit writer.
#[derive(Clone)]
pub struct SqliteAuditWriter {
    pool: SqlitePool,
}

impl SqliteAuditWriter {
    /// Opens a SQLite audit writer and ensures the schema exists.
    pub async fn connect(database_url: &str) -> Result<Self, GatewayError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .map_err(map_audit_error)?;

        query(include_str!("migrations/0001_audit_events.sql"))
            .execute(&pool)
            .await
            .map_err(map_audit_error)?;

        Ok(Self { pool })
    }

    /// Appends one audit event.
    pub async fn append(&self, event: &AuditEvent) -> Result<(), GatewayError> {
        let payload = serde_json::to_string(event).map_err(|_| {
            GatewayError::new(
                ErrorCode::AuditWriteFailed,
                "Failed to serialize audit event",
                true,
                Some("Inspect audit serialization".to_string()),
            )
        })?;

        query(
            "INSERT INTO audit_events (event_id, event_type, timestamp, payload_json) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(event.event_id.as_uuid().to_string())
        .bind(format!("{:?}", event.event_type))
        .bind(event.timestamp.unix_timestamp())
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(map_audit_error)?;

        Ok(())
    }

    /// Returns recent audit events, newest first.
    pub async fn tail(&self, request: AuditTailRequest) -> Result<AuditTail, GatewayError> {
        let rows = query(
            "SELECT sequence_id, payload_json FROM audit_events ORDER BY sequence_id DESC LIMIT ?1",
        )
        .bind(i64::from(request.normalized_limit()))
        .fetch_all(&self.pool)
        .await
        .map_err(map_audit_error)?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let sequence_id = row
                .try_get::<i64, _>("sequence_id")
                .map_err(map_audit_error)?;
            let payload_json = row
                .try_get::<String, _>("payload_json")
                .map_err(map_audit_error)?;
            let event = serde_json::from_str::<AuditEvent>(&payload_json).map_err(|_| {
                GatewayError::new(
                    ErrorCode::AuditWriteFailed,
                    "Failed to deserialize audit event",
                    true,
                    Some("Inspect local audit storage".to_string()),
                )
            })?;
            events.push(AuditTailRecord { sequence_id, event });
        }

        Ok(AuditTail { events })
    }

    /// Exports recent audit events as redacted JSONL.
    pub async fn export_jsonl(
        &self,
        request: AuditTailRequest,
    ) -> Result<AuditExport, GatewayError> {
        let limit = request.normalized_limit();
        let tail = self.tail(request).await?;
        export_audit_tail_jsonl(&tail, limit)
    }
}

fn map_audit_error(_error: SqlxError) -> GatewayError {
    GatewayError::new(
        ErrorCode::AuditWriteFailed,
        "Audit storage operation failed",
        true,
        Some("Check local audit SQLite storage".to_string()),
    )
}
