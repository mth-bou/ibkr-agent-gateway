//! SQLite append-only audit persistence.

use super::account_hash::AuditHmacKey;
use super::query::{AuditChainVerifyReport, AuditTail, AuditTailRecord, AuditTailRequest};
use super::redaction::{hmac_sha256_hex, scrub_audit_metadata, sha256_hex};
use super::{
    event::AuditEvent,
    export::{AuditExport, export_audit_tail_jsonl},
};
use crate::internal::approval::{ApprovalRecord, ApprovalStatus};
use crate::internal::domain::{
    AccountId, BracketOrderPreview, BrokerOrderId, CurrencyCode, ErrorCode, GatewayError, Money,
    OrderPreview, OrderPreviewId, ValidatedOrder, ValidatedOrderGroup,
};
use crate::internal::orders::{IdempotencyKey, LiveOrderLifecycleRecord, LiveOrderLifecycleStatus};
use serde::{Deserialize, Serialize};
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

/// Persisted order preview bound to the validated order it authorized.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrderPreviewRecord {
    /// Non-executable preview returned to the operator.
    pub preview: OrderPreview,
    /// Validated order candidate that must match approval at submit time.
    pub validated_order: ValidatedOrder,
}

/// Persisted bracket preview bound to its three validated legs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrderGroupRecord {
    /// Non-executable bracket preview returned to the operator.
    pub preview: BracketOrderPreview,
    /// Validated group candidate that must match approvals at submit time.
    pub validated_group: ValidatedOrderGroup,
}

/// Pending order idempotency record that may require broker-side recovery.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingOrderIdempotencyRecord {
    /// Idempotency key.
    pub idempotency_key: String,
    /// Canonical request hash.
    pub request_hash: String,
    /// Creation timestamp.
    pub created_at: i64,
    /// Last update timestamp.
    pub updated_at: i64,
    /// Broker recovery context persisted before the writer call.
    pub recovery_context: Option<OrderIdempotencyRecoveryContext>,
}

/// Pending live order that still needs broker lifecycle reconciliation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PendingLiveOrderRecord {
    /// Account id used for broker status polling.
    pub account_id: AccountId,
    /// Broker order id returned by the live writer.
    pub broker_order_id: BrokerOrderId,
    /// Last lifecycle status recorded by the gateway.
    pub last_status: LiveOrderLifecycleStatus,
    /// First time the order was added to the reconciliation backlog.
    pub created_at: i64,
    /// Last time the reconciler polled broker status for this order.
    pub last_polled_at: i64,
}

/// Server-side live submit counters derived from durable audit workflow state.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LiveRateCounts {
    /// Submitted live orders inside the configured frequency window.
    pub submitted_in_window: u32,
    /// Submitted live orders in the current local session history.
    pub submitted_in_session: u32,
    /// Submitted live notional in the requested currency.
    pub session_notional: Option<Money>,
}

/// Order workflow family for idempotency recovery payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderIdempotencyWorkflow {
    /// Paper order workflow.
    Paper,
    /// Live order workflow.
    Live,
}

/// Writer operation recorded before the broker boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderIdempotencyOperation {
    /// Submit operation; IBKR lookup uses the idempotency key as `cOID`.
    Submit,
    /// Cancel operation; lookup uses the known broker order id when present.
    Cancel,
    /// Modify operation; lookup uses the known broker order id.
    Modify,
}

/// Context needed to recover a pending order writer call after a crash.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrderIdempotencyRecoveryContext {
    /// Paper or live workflow.
    pub workflow: OrderIdempotencyWorkflow,
    /// Submit, cancel, or modify operation.
    pub operation: OrderIdempotencyOperation,
    /// Account id used for broker-side lookup.
    pub account_id: AccountId,
    /// Broker order id for cancel or modify recovery.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broker_order_id: Option<BrokerOrderId>,
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
        query(include_str!(
            "migrations/0003_live_orders_reconciliation.sql"
        ))
        .execute(&pool)
        .await
        .map_err(|err| map_audit_error(err, "apply live reconciliation schema migration"))?;
        query(include_str!("migrations/0004_order_groups.sql"))
            .execute(&pool)
            .await
            .map_err(|err| map_audit_error(err, "apply order group schema migration"))?;
        ensure_order_idempotency_columns(&pool).await?;

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

    /// Verifies the entire audit HMAC chain without returning event payloads.
    pub async fn verify_chain(&self) -> Result<AuditChainVerifyReport, GatewayError> {
        let rows = query(
            "SELECT sequence_id, event_id, payload_json, chain_hash FROM audit_events ORDER BY sequence_id ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "load audit events for chain verification"))?;

        let mut prev = String::new();
        let mut events_scanned = 0usize;
        for row in rows {
            events_scanned += 1;
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
                    "audit chain hash mismatch detected during verification"
                );
                return Ok(AuditChainVerifyReport {
                    events_scanned,
                    chain_valid: false,
                    first_break_at_sequence: Some(sequence_id),
                    first_break_event_id: Some(event_id),
                });
            }
            prev = stored_chain_hash;
        }

        Ok(AuditChainVerifyReport {
            events_scanned,
            chain_valid: true,
            first_break_at_sequence: None,
            first_break_event_id: None,
        })
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

    /// Persists a created order preview for later approval binding.
    pub async fn append_order_preview(
        &self,
        preview: &OrderPreview,
        validated_order: &ValidatedOrder,
    ) -> Result<(), GatewayError> {
        if preview.preview_id != validated_order.preview_id
            || preview.validated_order_id != validated_order.validated_order_id
        {
            return Err(GatewayError::new(
                ErrorCode::OrderValidationFailed,
                "Order preview and validated order identifiers do not match",
                false,
                Some("Create a fresh preview before approval".to_string()),
            ));
        }

        let record = OrderPreviewRecord {
            preview: preview.clone(),
            validated_order: validated_order.clone(),
        };
        let payload = serialize_audit_payload(&record, "serialize order preview record")?;
        let account_id_hash = self
            .audit_hmac_key
            .compute_account_id_hash(validated_order.account_id.as_str())?;
        query(
            "INSERT INTO order_preview_records (preview_id, validated_order_id, account_id_hash, expires_at, payload_json) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .bind(preview.preview_id.as_uuid().to_string())
        .bind(preview.validated_order_id.as_uuid().to_string())
        .bind(account_id_hash.as_str())
        .bind(preview.expires_at.unix_timestamp())
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "insert order preview record"))?;
        Ok(())
    }

    /// Persists a created bracket preview for later approval binding.
    pub async fn append_order_group(
        &self,
        preview: &BracketOrderPreview,
        validated_group: &ValidatedOrderGroup,
    ) -> Result<(), GatewayError> {
        if preview.group_id != validated_group.group_id
            || preview.parent.preview_id != validated_group.parent.preview_id
            || preview.take_profit.preview_id != validated_group.take_profit.preview_id
            || preview.stop_loss.preview_id != validated_group.stop_loss.preview_id
        {
            return Err(GatewayError::new(
                ErrorCode::OrderValidationFailed,
                "Bracket preview and validated group identifiers do not match",
                false,
                Some("Create a fresh bracket preview before approval".to_string()),
            ));
        }

        let record = OrderGroupRecord {
            preview: preview.clone(),
            validated_group: validated_group.clone(),
        };
        let payload = serialize_audit_payload(&record, "serialize order group record")?;
        let account_id_hash = self
            .audit_hmac_key
            .compute_account_id_hash(validated_group.account_id.as_str())?;
        query(
            "INSERT OR REPLACE INTO order_groups (group_id, account_id, payload_json, created_at) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(preview.group_id.as_uuid().to_string())
        .bind(account_id_hash.as_str())
        .bind(payload)
        .bind(time::OffsetDateTime::now_utc().unix_timestamp())
        .execute(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "insert order group record"))?;
        Ok(())
    }

    /// Loads a bracket group by the exact set of leg preview ids.
    pub async fn load_order_group_for_previews(
        &self,
        parent_preview_id: &OrderPreviewId,
        take_profit_preview_id: &OrderPreviewId,
        stop_loss_preview_id: &OrderPreviewId,
    ) -> Result<Option<OrderGroupRecord>, GatewayError> {
        let rows = query("SELECT payload_json FROM order_groups ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| map_audit_error(err, "load order group records"))?;

        for row in rows {
            let payload_json = row
                .try_get::<String, _>("payload_json")
                .map_err(|err| map_audit_error(err, "decode order group payload_json"))?;
            let record =
                serde_json::from_str::<OrderGroupRecord>(&payload_json).map_err(|err| {
                    error!(
                        target: "audit",
                        error = %err,
                        "failed to deserialize order group record"
                    );
                    GatewayError::new(
                        ErrorCode::AuditWriteFailed,
                        "Failed to deserialize order group record",
                        true,
                        Some("Inspect local bracket preview storage".to_string()),
                    )
                })?;
            if &record.preview.parent.preview_id == parent_preview_id
                && &record.preview.take_profit.preview_id == take_profit_preview_id
                && &record.preview.stop_loss.preview_id == stop_loss_preview_id
            {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }

    /// Loads a persisted order preview by id.
    pub async fn load_order_preview(
        &self,
        preview_id: &OrderPreviewId,
    ) -> Result<Option<OrderPreviewRecord>, GatewayError> {
        let row = query("SELECT payload_json FROM order_preview_records WHERE preview_id = ?1")
            .bind(preview_id.as_uuid().to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| map_audit_error(err, "load order preview record"))?;

        let Some(row) = row else {
            return Ok(None);
        };
        let payload_json = row
            .try_get::<String, _>("payload_json")
            .map_err(|err| map_audit_error(err, "decode order preview payload_json"))?;
        serde_json::from_str::<OrderPreviewRecord>(&payload_json)
            .map(Some)
            .map_err(|err| {
                error!(
                    target: "audit",
                    error = %err,
                    preview_id = %preview_id.as_uuid(),
                    "failed to deserialize order preview record"
                );
                GatewayError::new(
                    ErrorCode::AuditWriteFailed,
                    "Failed to deserialize order preview record",
                    true,
                    Some("Inspect local preview storage".to_string()),
                )
            })
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

    /// Marks an approval as consumed after a successful submit.
    pub async fn mark_approval_consumed(
        &self,
        approval: &ApprovalRecord,
    ) -> Result<(), GatewayError> {
        let mut consumed = approval.clone();
        consumed.status = ApprovalStatus::Consumed;
        let payload = serialize_audit_payload(&consumed, "serialize consumed approval record")?;
        query("UPDATE approval_records SET status = ?1, payload_json = ?2 WHERE approval_id = ?3")
            .bind(approval_status_name(consumed.status))
            .bind(payload)
            .bind(consumed.approval_id.as_uuid().to_string())
            .execute(&self.pool)
            .await
            .map_err(|err| map_audit_error(err, "mark approval consumed"))?;
        Ok(())
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
            "SELECT request_hash, status, payload_json FROM order_idempotency_records WHERE idempotency_key = ?1",
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
        let status = row
            .try_get::<String, _>("status")
            .map_err(|err| map_audit_error(err, "decode idempotency status"))?;
        match status.as_str() {
            "submitted" => {}
            "pending_writer" => {
                return Err(GatewayError::new(
                    ErrorCode::PaperIdempotencyConflict,
                    "Idempotency key has a pending broker writer call",
                    true,
                    Some("Run idempotency recovery before retrying this key".to_string()),
                ));
            }
            "failed_after_writer" => {
                return Err(GatewayError::new(
                    ErrorCode::BrokerBackendUnavailable,
                    "Previous broker writer call failed before completion was recorded",
                    true,
                    Some("Inspect the broker-side order state before retrying".to_string()),
                ));
            }
            _ => {
                return Err(GatewayError::new(
                    ErrorCode::AuditWriteFailed,
                    "Unknown order idempotency status",
                    true,
                    Some("Inspect local idempotency storage".to_string()),
                ));
            }
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

    /// Stores a pending writer call before the broker adapter is invoked.
    pub async fn insert_order_pending(
        &self,
        idempotency_key: &IdempotencyKey,
        request_hash: &str,
    ) -> Result<(), GatewayError> {
        self.insert_order_pending_with_context(idempotency_key, request_hash, None)
            .await
    }

    /// Stores a pending writer call with broker recovery context.
    pub async fn insert_order_pending_with_context(
        &self,
        idempotency_key: &IdempotencyKey,
        request_hash: &str,
        recovery_context: Option<&OrderIdempotencyRecoveryContext>,
    ) -> Result<(), GatewayError> {
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let payload_json = match recovery_context {
            Some(context) => {
                serialize_audit_payload(context, "serialize idempotency recovery context")?
            }
            None => "{}".to_string(),
        };
        query(
            "INSERT INTO order_idempotency_records (idempotency_key, request_hash, result_hash, created_at, payload_json, status, updated_at, failure_json) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, NULL)",
        )
        .bind(idempotency_key.as_str())
        .bind(request_hash)
        .bind(now)
        .bind(payload_json)
        .bind("pending_writer")
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "insert pending order idempotency record"))?;
        Ok(())
    }

    /// Removes a pending writer call when validation failed before the writer.
    pub async fn delete_order_pending(
        &self,
        idempotency_key: &IdempotencyKey,
        request_hash: &str,
    ) -> Result<(), GatewayError> {
        query(
            "DELETE FROM order_idempotency_records WHERE idempotency_key = ?1 AND request_hash = ?2 AND status = ?3",
        )
        .bind(idempotency_key.as_str())
        .bind(request_hash)
        .bind("pending_writer")
        .execute(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "delete pending order idempotency record"))?;
        Ok(())
    }

    /// Marks a pending writer call as failed after reaching the writer.
    pub async fn mark_order_failed_after_writer(
        &self,
        idempotency_key: &IdempotencyKey,
        request_hash: &str,
        error: &GatewayError,
    ) -> Result<(), GatewayError> {
        let failure_json = serialize_audit_payload(error, "serialize writer failure")?;
        query(
            "UPDATE order_idempotency_records SET status = ?1, updated_at = ?2, failure_json = ?3 WHERE idempotency_key = ?4 AND request_hash = ?5 AND status = ?6",
        )
        .bind("failed_after_writer")
        .bind(time::OffsetDateTime::now_utc().unix_timestamp())
        .bind(failure_json)
        .bind(idempotency_key.as_str())
        .bind(request_hash)
        .bind("pending_writer")
        .execute(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "mark order idempotency failed after writer"))?;
        Ok(())
    }

    /// Lists pending writer calls that need broker-side recovery.
    pub async fn pending_order_idempotency_records(
        &self,
    ) -> Result<Vec<PendingOrderIdempotencyRecord>, GatewayError> {
        let rows = query(
            "SELECT idempotency_key, request_hash, payload_json, created_at, updated_at FROM order_idempotency_records WHERE status = ?1 ORDER BY created_at ASC",
        )
        .bind("pending_writer")
        .fetch_all(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "list pending order idempotency records"))?;

        rows.into_iter()
            .map(|row| {
                Ok(PendingOrderIdempotencyRecord {
                    idempotency_key: row
                        .try_get::<String, _>("idempotency_key")
                        .map_err(|err| map_audit_error(err, "decode pending idempotency key"))?,
                    request_hash: row
                        .try_get::<String, _>("request_hash")
                        .map_err(|err| map_audit_error(err, "decode pending request hash"))?,
                    created_at: row
                        .try_get::<i64, _>("created_at")
                        .map_err(|err| map_audit_error(err, "decode pending created_at"))?,
                    updated_at: row
                        .try_get::<i64, _>("updated_at")
                        .map_err(|err| map_audit_error(err, "decode pending updated_at"))?,
                    recovery_context: decode_recovery_context(
                        &row.try_get::<String, _>("payload_json")
                            .map_err(|err| map_audit_error(err, "decode pending payload_json"))?,
                    )?,
                })
            })
            .collect()
    }

    /// Stores a completed order workflow result for idempotent replay.
    pub async fn insert_order_idempotency(
        &self,
        idempotency_key: &IdempotencyKey,
        request_hash: &str,
        payload: &serde_json::Value,
    ) -> Result<(), GatewayError> {
        self.complete_order_idempotency(idempotency_key, request_hash, payload)
            .await
    }

    /// Completes a pending or fresh order idempotency record.
    pub async fn complete_order_idempotency(
        &self,
        idempotency_key: &IdempotencyKey,
        request_hash: &str,
        payload: &serde_json::Value,
    ) -> Result<(), GatewayError> {
        let payload_json = serialize_audit_payload(payload, "serialize idempotency payload")?;
        let result_hash = sha256_hex(payload_json.as_bytes());
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let row =
            query("SELECT request_hash FROM order_idempotency_records WHERE idempotency_key = ?1")
                .bind(idempotency_key.as_str())
                .fetch_optional(&self.pool)
                .await
                .map_err(|err| {
                    map_audit_error(err, "load order idempotency record for completion")
                })?;
        if let Some(row) = row {
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
            query(
                "UPDATE order_idempotency_records SET result_hash = ?1, payload_json = ?2, status = ?3, updated_at = ?4, failure_json = NULL WHERE idempotency_key = ?5",
            )
            .bind(result_hash)
            .bind(payload_json)
            .bind("submitted")
            .bind(now)
            .bind(idempotency_key.as_str())
            .execute(&self.pool)
            .await
            .map_err(|err| map_audit_error(err, "complete order idempotency record"))?;
        } else {
            query(
                "INSERT INTO order_idempotency_records (idempotency_key, request_hash, result_hash, created_at, payload_json, status, updated_at, failure_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
            )
            .bind(idempotency_key.as_str())
            .bind(request_hash)
            .bind(result_hash)
            .bind(now)
            .bind(payload_json)
            .bind("submitted")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|err| map_audit_error(err, "insert completed order idempotency record"))?;
        }
        Ok(())
    }

    /// Adds or updates one live order in the reconciliation backlog.
    ///
    /// Terminal lifecycle records are removed instead of kept for polling.
    pub async fn upsert_live_order_pending(
        &self,
        lifecycle: &LiveOrderLifecycleRecord,
    ) -> Result<(), GatewayError> {
        if lifecycle.status.is_terminal() {
            return self
                .remove_live_order_pending(&lifecycle.account_id, &lifecycle.broker_order_id)
                .await;
        }

        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        query(
            "INSERT INTO live_orders_pending (account_id, broker_order_id, last_status, created_at, last_polled_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(account_id, broker_order_id) DO UPDATE SET
                last_status = excluded.last_status,
                last_polled_at = excluded.last_polled_at",
        )
        .bind(lifecycle.account_id.as_str())
        .bind(lifecycle.broker_order_id.as_str())
        .bind(live_status_name(lifecycle.status))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "upsert pending live order"))?;
        Ok(())
    }

    /// Removes one live order from the reconciliation backlog.
    pub async fn remove_live_order_pending(
        &self,
        account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
    ) -> Result<(), GatewayError> {
        query("DELETE FROM live_orders_pending WHERE account_id = ?1 AND broker_order_id = ?2")
            .bind(account_id.as_str())
            .bind(broker_order_id.as_str())
            .execute(&self.pool)
            .await
            .map_err(|err| map_audit_error(err, "remove pending live order"))?;
        Ok(())
    }

    /// Lists live orders that are still non-terminal and should be polled.
    pub async fn pending_live_orders(&self) -> Result<Vec<PendingLiveOrderRecord>, GatewayError> {
        let rows = query(
            "SELECT account_id, broker_order_id, last_status, created_at, last_polled_at
             FROM live_orders_pending
             ORDER BY last_polled_at ASC, created_at ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "list pending live orders"))?;

        rows.into_iter()
            .map(|row| {
                let account_id = row
                    .try_get::<String, _>("account_id")
                    .map_err(|err| map_audit_error(err, "decode live order account_id"))?;
                let broker_order_id = row
                    .try_get::<String, _>("broker_order_id")
                    .map_err(|err| map_audit_error(err, "decode live order broker_order_id"))?;
                let last_status = row
                    .try_get::<String, _>("last_status")
                    .map_err(|err| map_audit_error(err, "decode live order last_status"))?;
                Ok(PendingLiveOrderRecord {
                    account_id: AccountId::new(account_id).ok_or_else(|| {
                        GatewayError::new(
                            ErrorCode::AuditWriteFailed,
                            "Stored pending live order account id is invalid",
                            true,
                            Some("Inspect local live order reconciliation storage".to_string()),
                        )
                    })?,
                    broker_order_id: BrokerOrderId::new(broker_order_id).ok_or_else(|| {
                        GatewayError::new(
                            ErrorCode::AuditWriteFailed,
                            "Stored pending live order broker order id is invalid",
                            true,
                            Some("Inspect local live order reconciliation storage".to_string()),
                        )
                    })?,
                    last_status: parse_live_status(&last_status)?,
                    created_at: row
                        .try_get::<i64, _>("created_at")
                        .map_err(|err| map_audit_error(err, "decode live order created_at"))?,
                    last_polled_at: row
                        .try_get::<i64, _>("last_polled_at")
                        .map_err(|err| map_audit_error(err, "decode live order last_polled_at"))?,
                })
            })
            .collect()
    }

    /// Rebuilds the live reconciliation backlog from completed live lifecycle
    /// idempotency records.
    pub async fn rebuild_live_order_pending_from_idempotency(&self) -> Result<usize, GatewayError> {
        let rows = query(
            "SELECT payload_json FROM order_idempotency_records WHERE status = ?1 ORDER BY created_at ASC, updated_at ASC",
        )
        .bind("submitted")
        .fetch_all(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "scan completed order idempotency records"))?;

        for row in rows {
            let payload_json = row
                .try_get::<String, _>("payload_json")
                .map_err(|err| map_audit_error(err, "decode idempotency payload_json"))?;
            let Ok(lifecycle) = serde_json::from_str::<LiveOrderLifecycleRecord>(&payload_json)
            else {
                continue;
            };
            self.upsert_live_order_pending(&lifecycle).await?;
        }
        Ok(self.pending_live_orders().await?.len())
    }

    /// Counts prior live submit records for server-side live rate limits.
    pub async fn live_rate_counts(
        &self,
        account_id: &AccountId,
        window_seconds: Option<u64>,
    ) -> Result<LiveRateCounts, GatewayError> {
        self.live_rate_counts_with_session_currency(account_id, window_seconds, None)
            .await
    }

    /// Counts prior live submit records and sums submitted notional for one currency.
    pub async fn live_rate_counts_with_session_currency(
        &self,
        account_id: &AccountId,
        window_seconds: Option<u64>,
        session_currency: Option<&CurrencyCode>,
    ) -> Result<LiveRateCounts, GatewayError> {
        let rows = query(
            "SELECT created_at, payload_json FROM order_idempotency_records WHERE status = ?1 ORDER BY created_at ASC",
        )
        .bind("submitted")
        .fetch_all(&self.pool)
        .await
        .map_err(|err| map_audit_error(err, "scan live order rate counters"))?;
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let window_start =
            window_seconds.and_then(|seconds| i64::try_from(seconds).ok().map(|s| now - s));
        let mut counts = LiveRateCounts::default();
        for row in rows {
            let created_at = row
                .try_get::<i64, _>("created_at")
                .map_err(|err| map_audit_error(err, "decode idempotency created_at"))?;
            let payload_json = row
                .try_get::<String, _>("payload_json")
                .map_err(|err| map_audit_error(err, "decode idempotency payload_json"))?;
            let Ok(lifecycle) = serde_json::from_str::<LiveOrderLifecycleRecord>(&payload_json)
            else {
                continue;
            };
            if lifecycle.account_id != *account_id || !counts_as_live_submit(lifecycle.status) {
                continue;
            }
            counts.submitted_in_session = counts.submitted_in_session.saturating_add(1);
            if let (Some(currency), Some(notional)) = (session_currency, lifecycle.notional)
                && &notional.currency == currency
            {
                add_session_notional(&mut counts, notional);
            }
            if window_start.is_none_or(|start| created_at >= start) {
                counts.submitted_in_window = counts.submitted_in_window.saturating_add(1);
            }
        }
        Ok(counts)
    }
}

fn add_session_notional(counts: &mut LiveRateCounts, notional: Money) {
    match &mut counts.session_notional {
        Some(total) => total.amount += notional.amount,
        None => counts.session_notional = Some(notional),
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

fn decode_recovery_context(
    payload_json: &str,
) -> Result<Option<OrderIdempotencyRecoveryContext>, GatewayError> {
    if payload_json.trim().is_empty() || payload_json.trim() == "{}" {
        return Ok(None);
    }
    serde_json::from_str::<OrderIdempotencyRecoveryContext>(payload_json)
        .map(Some)
        .map_err(|err| {
            error!(
                target: "audit",
                error = %err,
                "failed to deserialize idempotency recovery context"
            );
            GatewayError::new(
                ErrorCode::AuditWriteFailed,
                "Failed to deserialize idempotency recovery context",
                true,
                Some("Inspect local idempotency storage".to_string()),
            )
        })
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

async fn ensure_order_idempotency_columns(pool: &SqlitePool) -> Result<(), GatewayError> {
    for (name, definition) in [
        ("status", "TEXT NOT NULL DEFAULT 'submitted'"),
        ("updated_at", "INTEGER"),
        ("failure_json", "TEXT"),
    ] {
        if !sqlite_table_has_column(pool, "order_idempotency_records", name).await? {
            let sql =
                format!("ALTER TABLE order_idempotency_records ADD COLUMN {name} {definition}");
            query(&sql)
                .execute(pool)
                .await
                .map_err(|err| map_audit_error(err, "add order idempotency column"))?;
        }
    }

    query("UPDATE order_idempotency_records SET updated_at = created_at WHERE updated_at IS NULL")
        .execute(pool)
        .await
        .map_err(|err| map_audit_error(err, "backfill order idempotency updated_at"))?;
    Ok(())
}

async fn sqlite_table_has_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
) -> Result<bool, GatewayError> {
    let pragma = format!("PRAGMA table_info({table})");
    let rows = query(&pragma)
        .fetch_all(pool)
        .await
        .map_err(|err| map_audit_error(err, "inspect sqlite table columns"))?;
    for row in rows {
        let name = row
            .try_get::<String, _>("name")
            .map_err(|err| map_audit_error(err, "decode sqlite column name"))?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
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

const fn live_status_name(status: LiveOrderLifecycleStatus) -> &'static str {
    match status {
        LiveOrderLifecycleStatus::Submitted => "submitted",
        LiveOrderLifecycleStatus::Open => "open",
        LiveOrderLifecycleStatus::PendingCancel => "pending_cancel",
        LiveOrderLifecycleStatus::Filled => "filled",
        LiveOrderLifecycleStatus::Cancelled => "cancelled",
        LiveOrderLifecycleStatus::Refused => "refused",
    }
}

fn parse_live_status(status: &str) -> Result<LiveOrderLifecycleStatus, GatewayError> {
    match status {
        "submitted" => Ok(LiveOrderLifecycleStatus::Submitted),
        "open" => Ok(LiveOrderLifecycleStatus::Open),
        "pending_cancel" => Ok(LiveOrderLifecycleStatus::PendingCancel),
        "filled" => Ok(LiveOrderLifecycleStatus::Filled),
        "cancelled" => Ok(LiveOrderLifecycleStatus::Cancelled),
        "refused" => Ok(LiveOrderLifecycleStatus::Refused),
        _ => Err(GatewayError::new(
            ErrorCode::AuditWriteFailed,
            "Stored pending live order status is invalid",
            true,
            Some("Inspect local live order reconciliation storage".to_string()),
        )),
    }
}

const fn counts_as_live_submit(status: LiveOrderLifecycleStatus) -> bool {
    matches!(
        status,
        LiveOrderLifecycleStatus::Submitted
            | LiveOrderLifecycleStatus::Open
            | LiveOrderLifecycleStatus::PendingCancel
            | LiveOrderLifecycleStatus::Filled
    )
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
