//! SQLite append-only audit persistence.

use crate::event::AuditEvent;
use ibkr_domain::{ErrorCode, GatewayError};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

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

        sqlx::query(include_str!("../migrations/0001_audit_events.sql"))
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

        sqlx::query(
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
}

fn map_audit_error(_error: sqlx::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::AuditWriteFailed,
        "Audit storage operation failed",
        true,
        Some("Check local audit SQLite storage".to_string()),
    )
}
