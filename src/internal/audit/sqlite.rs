//! SQLite append-only audit persistence.

use super::account_hash::AuditHmacKey;
use super::query::{AuditTail, AuditTailRecord, AuditTailRequest};
use super::redaction::{hmac_sha256_hex, scrub_audit_metadata, sha256_hex};
use super::{
    event::AuditEvent,
    export::{AuditExport, export_audit_tail_jsonl},
};
use crate::internal::approval::{ApprovalRecord, ApprovalStatus};
use crate::internal::domain::{ErrorCode, GatewayError};
use crate::internal::orders::IdempotencyKey;
use sqlx_core::{Error as SqlxError, query::query, row::Row};
use sqlx_sqlite::{SqlitePool, SqlitePoolOptions};
use std::mem;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, warn};

/// SQLite-backed audit writer.
///
/// Each appended event is bound to its predecessor via a chained HMAC-SHA256
/// computed under the audit HMAC key:
///
/// ```text
/// chain_hash = HMAC(key, prev_chain_hash || ":" || event_id || ":" || sha256(payload_json))
/// ```
///
/// Deleting or reordering rows in SQLite breaks the chain on the next
/// [`tail`](Self::tail) verification call (see [`tail_verified`](Self::tail_verified)).
#[derive(Clone)]
pub struct SqliteAuditWriter {
    pool: SqlitePool,
    audit_hmac_key: Arc<AuditHmacKey>,
    last_chain_hash: Arc<Mutex<String>>,
}

impl SqliteAuditWriter {
    /// Opens a SQLite audit writer and ensures the schema exists.
    ///
    /// The connection is configured for WAL journalling and `NORMAL`
    /// synchronous mode to reduce per-write fsync latency. WAL is durable
    /// across OS crashes; the trade-off versus `FULL` is that a hardware
    /// power-cut can cost the last in-flight transaction, which is
    /// acceptable for a local audit log.
    pub async fn connect(
        database_url: &str,
        audit_hmac_key: Arc<AuditHmacKey>,
    ) -> Result<Self, GatewayError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .map_err(|err| map_audit_error(err, "open audit sqlite pool"))?;

        query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await
            .map_err(|err| map_audit_error(err, "set journal_mode=WAL"))?;
        query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await
            .map_err(|err| map_audit_error(err, "set synchronous=NORMAL"))?;

        query(include_str!("migrations/0001_audit_events.sql"))
            .execute(&pool)
            .await
            .map_err(|err| map_audit_error(err, "apply audit schema migration"))?;
        query(include_str!("migrations/0002_paper_orders.sql"))
            .execute(&pool)
            .await
            .map_err(|err| map_audit_error(err, "apply order workflow schema migration"))?;

        let last_chain_hash = load_last_chain_hash(&pool).await?;

        Ok(Self {
            pool,
            audit_hmac_key,
            last_chain_hash: Arc::new(Mutex::new(last_chain_hash)),
        })
    }

    /// Appends one audit event.
    ///
    /// Sensitive keys in `event.metadata` are replaced with `"[REDACTED]"`
    /// before the row is written, and a `RedactionRecord` is appended to the
    /// event's redaction trail. The persisted row also carries a `chain_hash`
    /// that binds it to the previous row so deletions or reorderings become
    /// detectable on tail verification.
    pub async fn append(&self, event: &AuditEvent) -> Result<(), GatewayError> {
        let event = scrub_event_for_persistence(event);
        let event = &event;
        let payload = serde_json::to_string(event).map_err(|err| {
            error!(
                target: "audit",
                error = %err,
                event_id = %event.event_id.as_uuid(),
                "failed to serialize audit event"
            );
            GatewayError::new(
                ErrorCode::AuditWriteFailed,
                "Failed to serialize audit event",
                true,
                Some("Inspect audit serialization".to_string()),
            )
        })?;

        let mut last_chain_hash = self.last_chain_hash.lock().await;
        query("BEGIN IMMEDIATE")
            .execute(&self.pool)
            .await
            .map_err(|err| map_audit_error(err, "begin audit append transaction"))?;

        let current_chain_hash = match load_last_chain_hash(&self.pool).await {
            Ok(value) => value,
            Err(error) => {
                rollback_audit_transaction(&self.pool).await;
                return Err(error);
            }
        };
        let chain_hash = match compute_chain_hash(
            &self.audit_hmac_key,
            &current_chain_hash,
            event.event_id.as_uuid().to_string().as_str(),
            &payload,
        ) {
            Ok(value) => value,
            Err(error) => {
                rollback_audit_transaction(&self.pool).await;
                return Err(error);
            }
        };

        if let Err(error) = query(
            "INSERT INTO audit_events (event_id, event_type, timestamp, payload_json, chain_hash) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(event.event_id.as_uuid().to_string())
        .bind(event.event_type.as_str())
        .bind(event.timestamp.unix_timestamp())
        .bind(payload)
        .bind(&chain_hash)
        .execute(&self.pool)
        .await
        {
            rollback_audit_transaction(&self.pool).await;
            return Err(map_audit_error(error, "insert audit event"));
        }

        if let Err(error) = query("COMMIT").execute(&self.pool).await {
            rollback_audit_transaction(&self.pool).await;
            return Err(map_audit_error(error, "commit audit append transaction"));
        }

        *last_chain_hash = chain_hash;

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
        .map_err(|err| map_audit_error(err, "tail audit events"))?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let sequence_id = row
                .try_get::<i64, _>("sequence_id")
                .map_err(|err| map_audit_error(err, "decode audit sequence_id"))?;
            let payload_json = row
                .try_get::<String, _>("payload_json")
                .map_err(|err| map_audit_error(err, "decode audit payload_json"))?;
            let event = serde_json::from_str::<AuditEvent>(&payload_json).map_err(|err| {
                error!(
                    target: "audit",
                    error = %err,
                    sequence_id,
                    "failed to deserialize audit event"
                );
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

    /// Returns recent audit events and verifies the chain hash of every row
    /// matches the recomputed HMAC. Returns an error on the first break.
    pub async fn tail_verified(
        &self,
        request: AuditTailRequest,
    ) -> Result<AuditTail, GatewayError> {
        let limit = i64::from(request.normalized_limit());
        let rows = query(
            "SELECT sequence_id, event_id, payload_json, chain_hash FROM audit_events ORDER BY sequence_id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "tail audit events for verification"))?;

        let mut prev = String::new();
        let mut records: Vec<AuditTailRecord> = Vec::with_capacity(rows.len());
        for row in rows {
            let sequence_id = row
                .try_get::<i64, _>("sequence_id")
                .map_err(|err| map_audit_error(err, "decode audit sequence_id"))?;
            let event_id = row
                .try_get::<String, _>("event_id")
                .map_err(|err| map_audit_error(err, "decode audit event_id"))?;
            let payload_json = row
                .try_get::<String, _>("payload_json")
                .map_err(|err| map_audit_error(err, "decode audit payload_json"))?;
            let stored_chain_hash = row
                .try_get::<String, _>("chain_hash")
                .map_err(|err| map_audit_error(err, "decode audit chain_hash"))?;

            let expected =
                compute_chain_hash(&self.audit_hmac_key, &prev, &event_id, &payload_json)?;
            if expected != stored_chain_hash {
                warn!(
                    target: "audit",
                    sequence_id,
                    "audit chain hash mismatch — log may have been tampered with"
                );
                return Err(GatewayError::new(
                    ErrorCode::AuditWriteFailed,
                    "Audit chain hash mismatch detected",
                    false,
                    Some("Investigate the audit storage for tampering".to_string()),
                ));
            }

            let event = serde_json::from_str::<AuditEvent>(&payload_json).map_err(|err| {
                error!(
                    target: "audit",
                    error = %err,
                    sequence_id,
                    "failed to deserialize audit event"
                );
                GatewayError::new(
                    ErrorCode::AuditWriteFailed,
                    "Failed to deserialize audit event",
                    true,
                    Some("Inspect local audit storage".to_string()),
                )
            })?;
            records.push(AuditTailRecord { sequence_id, event });
            prev = stored_chain_hash;
        }

        let take = usize::try_from(limit.max(0)).unwrap_or(records.len());
        records.reverse();
        records.truncate(take);
        Ok(AuditTail { events: records })
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

    /// Persists one approval record for later paper/live submit validation.
    pub async fn append_approval(&self, approval: &ApprovalRecord) -> Result<(), GatewayError> {
        let payload = serialize_audit_payload(approval, "serialize approval record")?;
        let account_id_hash = self
            .audit_hmac_key
            .compute_account_id_hash(approval.account_id.as_str())?;
        query(
            "INSERT INTO approval_records (approval_id, preview_id, account_id_hash, status, approved_by, approved_at, expires_at, payload_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(approval.approval_id.as_uuid().to_string())
        .bind(approval.preview_id.as_uuid().to_string())
        .bind(account_id_hash.as_str())
        .bind(approval_status_name(approval.status))
        .bind(approval.approved_by.as_str())
        .bind(approval.approved_at.map(|timestamp| timestamp.unix_timestamp()))
        .bind(approval.expires_at.unix_timestamp())
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "insert approval record"))?;
        Ok(())
    }

    /// Loads an approval record by id.
    pub async fn load_approval(
        &self,
        approval_id: &crate::internal::approval::ApprovalId,
    ) -> Result<Option<ApprovalRecord>, GatewayError> {
        let row = query("SELECT payload_json FROM approval_records WHERE approval_id = ?1")
            .bind(approval_id.as_uuid().to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| map_audit_error(err, "load approval record"))?;

        let Some(row) = row else {
            return Ok(None);
        };
        let payload_json = row
            .try_get::<String, _>("payload_json")
            .map_err(|err| map_audit_error(err, "decode approval payload_json"))?;
        serde_json::from_str::<ApprovalRecord>(&payload_json)
            .map(Some)
            .map_err(|err| {
                error!(
                    target: "audit",
                    error = %err,
                    approval_id = %approval_id.as_uuid(),
                    "failed to deserialize approval record"
                );
                GatewayError::new(
                    ErrorCode::AuditWriteFailed,
                    "Failed to deserialize approval record",
                    true,
                    Some("Inspect local approval storage".to_string()),
                )
            })
    }

    /// Returns a stored idempotent result when the key and request hash match.
    ///
    /// A matching key with a different hash is refused as a conflict.
    pub async fn replay_order_idempotency(
        &self,
        idempotency_key: &IdempotencyKey,
        request_hash: &str,
    ) -> Result<Option<serde_json::Value>, GatewayError> {
        let row = query(
            "SELECT request_hash, payload_json FROM order_idempotency_records WHERE idempotency_key = ?1",
        )
        .bind(idempotency_key.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "load order idempotency record"))?;

        let Some(row) = row else {
            return Ok(None);
        };
        let stored_hash = row
            .try_get::<String, _>("request_hash")
            .map_err(|err| map_audit_error(err, "decode idempotency request_hash"))?;
        if stored_hash != request_hash {
            return Err(GatewayError::new(
                ErrorCode::PaperIdempotencyConflict,
                "Idempotency key conflicts with a previous request",
                false,
                Some("Use a new idempotency key for a different request".to_string()),
            ));
        }
        let payload_json = row
            .try_get::<String, _>("payload_json")
            .map_err(|err| map_audit_error(err, "decode idempotency payload_json"))?;
        serde_json::from_str::<serde_json::Value>(&payload_json)
            .map(Some)
            .map_err(|err| {
                error!(
                    target: "audit",
                    error = %err,
                    idempotency_key = idempotency_key.as_str(),
                    "failed to deserialize idempotency result"
                );
                GatewayError::new(
                    ErrorCode::AuditWriteFailed,
                    "Failed to deserialize idempotency result",
                    true,
                    Some("Inspect local idempotency storage".to_string()),
                )
            })
    }

    /// Stores a completed order workflow result for idempotent replay.
    pub async fn insert_order_idempotency(
        &self,
        idempotency_key: &IdempotencyKey,
        request_hash: &str,
        payload: &serde_json::Value,
    ) -> Result<(), GatewayError> {
        let payload_json = serialize_audit_payload(payload, "serialize idempotency payload")?;
        let result_hash = sha256_hex(payload_json.as_bytes());
        query(
            "INSERT INTO order_idempotency_records (idempotency_key, request_hash, result_hash, created_at, payload_json) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(idempotency_key.as_str())
        .bind(request_hash)
        .bind(result_hash)
        .bind(time::OffsetDateTime::now_utc().unix_timestamp())
        .bind(payload_json)
        .execute(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "insert order idempotency record"))?;
        Ok(())
    }
}

async fn rollback_audit_transaction(pool: &SqlitePool) {
    if let Err(error) = query("ROLLBACK").execute(pool).await {
        warn!(
            target: "audit",
            error = %error,
            "failed to roll back audit append transaction"
        );
    }
}

/// Reads the most recent `chain_hash` from `audit_events`, returning empty when
/// the table is empty. Used to seed the writer at construction so chained
/// appends survive process restarts on existing databases.
async fn load_last_chain_hash(pool: &SqlitePool) -> Result<String, GatewayError> {
    let row = query("SELECT chain_hash FROM audit_events ORDER BY sequence_id DESC LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(|err| map_audit_error(err, "load last chain_hash"))?;

    let Some(row) = row else {
        return Ok(String::new());
    };
    row.try_get::<String, _>("chain_hash")
        .map_err(|err| map_audit_error(err, "decode last chain_hash"))
}

/// Computes `HMAC(key, prev_chain_hash || ":" || event_id || ":" || sha256(payload_json))`.
fn compute_chain_hash(
    key: &AuditHmacKey,
    prev_chain_hash: &str,
    event_id: &str,
    payload_json: &str,
) -> Result<String, GatewayError> {
    let payload_digest = sha256_hex(payload_json.as_bytes());
    let material = format!("{prev_chain_hash}:{event_id}:{payload_digest}");
    hmac_sha256_hex(key.as_bytes(), material.as_bytes()).map_err(|err| {
        error!(
            target: "audit",
            error = %err,
            "failed to compute audit chain hash"
        );
        GatewayError::new(
            ErrorCode::AuditWriteFailed,
            "Failed to compute audit chain hash",
            true,
            Some("Retry the audit write".to_string()),
        )
    })
}

/// Returns a clone of `event` with sensitive metadata keys redacted in place
/// and matching redaction records appended. Allocations occur only when at
/// least one key needs to be scrubbed.
fn scrub_event_for_persistence(event: &AuditEvent) -> AuditEvent {
    let mut cloned = event.clone();
    let original = mem::take(&mut cloned.metadata);
    let (scrubbed, mut new_redactions) = scrub_audit_metadata(original);
    cloned.metadata = scrubbed;
    cloned.redactions.append(&mut new_redactions);
    cloned
}

fn serialize_audit_payload<T: serde::Serialize>(
    value: &T,
    operation: &str,
) -> Result<String, GatewayError> {
    serde_json::to_string(value).map_err(|err| {
        error!(
            target: "audit",
            operation,
            error = %err,
            "failed to serialize audit-side payload"
        );
        GatewayError::new(
            ErrorCode::AuditWriteFailed,
            "Failed to serialize audit-side payload",
            true,
            Some("Inspect local audit serialization".to_string()),
        )
    })
}

const fn approval_status_name(status: ApprovalStatus) -> &'static str {
    match status {
        ApprovalStatus::Pending => "pending",
        ApprovalStatus::Approved => "approved",
        ApprovalStatus::Consumed => "consumed",
        ApprovalStatus::Expired => "expired",
        ApprovalStatus::Revoked => "revoked",
    }
}

fn map_audit_error(error: SqlxError, operation: &str) -> GatewayError {
    // The sqlx variant carries diagnostic detail (e.g. SQLITE_FULL, BUSY,
    // constraint violations) that is essential for production debugging. We
    // emit it via `tracing` before erasing it from the public error so the
    // operator-facing message stays redaction-safe.
    error!(
        target: "audit",
        operation,
        error = %error,
        error_debug = ?error,
        "audit storage operation failed"
    );
    GatewayError::new(
        ErrorCode::AuditWriteFailed,
        "Audit storage operation failed",
        true,
        Some("Check local audit SQLite storage".to_string()),
    )
}
