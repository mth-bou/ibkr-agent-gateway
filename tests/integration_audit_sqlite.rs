use ibkr_agent_gateway::testing::audit::{
    AuditDecision, AuditEvent, AuditEventType, AuditHmacKey, AuditResultStatus, SqliteAuditWriter,
};
use ibkr_agent_gateway::testing::domain::{
    AuditEventId, ErrorCode, GatewayError, LocalUserId, RequestId, SessionId,
};
use ibkr_agent_gateway::testing::orders::IdempotencyKey;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;
use time::OffsetDateTime;

#[tokio::test]
async fn appends_audit_event_to_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(key) = AuditHmacKey::ephemeral() else {
        return Err("ephemeral audit key generation must succeed".into());
    };
    let writer = SqliteAuditWriter::connect("sqlite::memory:", Arc::new(key)).await?;
    let Some(user_id) = LocalUserId::new("local-user") else {
        return Err("valid local user id rejected".into());
    };

    let event = AuditEvent {
        event_id: AuditEventId::new(),
        event_type: AuditEventType::ToolCalled,
        timestamp: OffsetDateTime::UNIX_EPOCH,
        user_id,
        session_id: SessionId::new(),
        request_id: RequestId::new(),
        account_id_hash: None,
        tool_name: Some("ibkr_health".to_string()),
        scopes: vec!["ibkr:health:read".to_string()],
        decision: AuditDecision::Allow,
        result_status: AuditResultStatus::Called,
        error_code: None,
        input_hash: None,
        output_hash: None,
        redactions: Vec::new(),
        metadata: BTreeMap::new(),
    };

    writer.append(&event).await?;
    Ok(())
}

#[tokio::test]
async fn chain_hash_detects_row_payload_tampering() -> Result<(), Box<dyn std::error::Error>> {
    use ibkr_agent_gateway::testing::audit::AuditTailRequest;
    use sqlx_core::query::query;
    use sqlx_sqlite::{SqlitePool, SqlitePoolOptions};

    let Ok(key) = AuditHmacKey::ephemeral() else {
        return Err("ephemeral audit key generation must succeed".into());
    };
    let key = Arc::new(key);
    // Use a shared file URI so the writer and our forensic tamper-helper
    // observe the same SQLite database.
    let url = "sqlite:file:audit_tamper?mode=memory&cache=shared";
    let writer = SqliteAuditWriter::connect(url, key.clone()).await?;
    let Some(user_id) = LocalUserId::new("local-user") else {
        return Err("valid local user id rejected".into());
    };

    for label in ["first", "second", "third"] {
        let event = AuditEvent {
            event_id: AuditEventId::new(),
            event_type: AuditEventType::ToolCalled,
            timestamp: OffsetDateTime::UNIX_EPOCH,
            user_id: user_id.clone(),
            session_id: SessionId::new(),
            request_id: RequestId::new(),
            account_id_hash: None,
            tool_name: Some(format!("ibkr_health_{label}")),
            scopes: vec!["ibkr:health:read".to_string()],
            decision: AuditDecision::Allow,
            result_status: AuditResultStatus::Called,
            error_code: None,
            input_hash: None,
            output_hash: None,
            redactions: Vec::new(),
            metadata: BTreeMap::new(),
        };
        writer.append(&event).await?;
    }

    // Verification must pass on an untouched chain.
    let verified = writer.tail_verified(AuditTailRequest::new(10)).await?;
    assert_eq!(verified.events.len(), 3);

    // Open a second handle to the same in-memory database and overwrite the
    // middle row's payload. This simulates an attacker editing the SQLite
    // file directly.
    let pool: SqlitePool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(url)
        .await?;
    query("UPDATE audit_events SET payload_json = ?1 WHERE sequence_id = 2")
        .bind("{\"forged\":true}")
        .execute(&pool)
        .await?;

    // Verification must now refuse the chain.
    let Err(error) = writer.tail_verified(AuditTailRequest::new(10)).await else {
        return Err("tampered row must break chain verification".into());
    };
    assert_eq!(error.code, ErrorCode::AuditWriteFailed);
    Ok(())
}

#[tokio::test]
async fn pending_order_idempotency_blocks_retry_until_completed()
-> Result<(), Box<dyn std::error::Error>> {
    let key = Arc::new(AuditHmacKey::ephemeral()?);
    let writer = SqliteAuditWriter::connect("sqlite::memory:", key).await?;
    let idempotency_key = IdempotencyKey::new("pending-submit-key")?;
    let request_hash = "stable-request-hash";

    writer
        .insert_order_pending(&idempotency_key, request_hash)
        .await?;
    let pending = writer.pending_order_idempotency_records().await?;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].idempotency_key, idempotency_key.as_str());

    let Err(pending_error) = writer
        .replay_order_idempotency(&idempotency_key, request_hash)
        .await
    else {
        return Err("pending writer record must block retry".into());
    };
    assert_eq!(pending_error.code, ErrorCode::PaperIdempotencyConflict);

    let payload = json!({ "broker_order_id": "paper-1", "status": "submitted" });
    writer
        .complete_order_idempotency(&idempotency_key, request_hash, &payload)
        .await?;
    assert!(writer.pending_order_idempotency_records().await?.is_empty());
    let replayed = writer
        .replay_order_idempotency(&idempotency_key, request_hash)
        .await?
        .ok_or("completed idempotency record must replay")?;
    assert_eq!(replayed, payload);
    Ok(())
}

#[tokio::test]
async fn failed_after_writer_record_blocks_retry_without_completion()
-> Result<(), Box<dyn std::error::Error>> {
    let key = Arc::new(AuditHmacKey::ephemeral()?);
    let writer = SqliteAuditWriter::connect("sqlite::memory:", key).await?;
    let idempotency_key = IdempotencyKey::new("failed-writer-key")?;
    let request_hash = "failed-request-hash";
    writer
        .insert_order_pending(&idempotency_key, request_hash)
        .await?;

    let writer_error = GatewayError::new(
        ErrorCode::BrokerBackendUnavailable,
        "writer unavailable",
        true,
        Some("check broker".to_string()),
    );
    writer
        .mark_order_failed_after_writer(&idempotency_key, request_hash, &writer_error)
        .await?;

    let Err(replay_error) = writer
        .replay_order_idempotency(&idempotency_key, request_hash)
        .await
    else {
        return Err("failed writer record must not replay or recall writer".into());
    };
    assert_eq!(replay_error.code, ErrorCode::BrokerBackendUnavailable);
    Ok(())
}
