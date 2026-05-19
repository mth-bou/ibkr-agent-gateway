use ibkr_agent_gateway::testing::approval::{ApprovalId, ApprovalRecord, ApprovalStatus};
use ibkr_agent_gateway::testing::audit::{
    AuditDecision, AuditEvent, AuditEventType, AuditHmacKey, AuditResultStatus, SqliteAuditWriter,
};
use ibkr_agent_gateway::testing::domain::{
    AccountId, AuditEventId, BrokerOrderId, ErrorCode, GatewayError, LocalUserId, OrderPreviewId,
    RequestId, SessionId,
};
use ibkr_agent_gateway::testing::orders::{
    IdempotencyKey, LiveOrderLifecycleRecord, LiveOrderLifecycleStatus,
};
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
    let verify_report = writer.verify_chain().await?;
    assert_eq!(verify_report.events_scanned, 3);
    assert!(verify_report.chain_valid);

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
    let verify_report = writer.verify_chain().await?;
    assert_eq!(verify_report.events_scanned, 2);
    assert!(!verify_report.chain_valid);
    assert_eq!(verify_report.first_break_at_sequence, Some(2));
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

#[tokio::test]
async fn live_reconciliation_backlog_and_rate_counts_rebuild_from_idempotency()
-> Result<(), Box<dyn std::error::Error>> {
    let key = Arc::new(AuditHmacKey::ephemeral()?);
    let writer = SqliteAuditWriter::connect("sqlite::memory:", key).await?;
    let account = AccountId::from_static("U1234567");
    let lifecycle = LiveOrderLifecycleRecord {
        account_id: account.clone(),
        broker_order_id: BrokerOrderId::from_static("live-1"),
        status: LiveOrderLifecycleStatus::Submitted,
        notional: None,
        execution_correlation: None,
        updated_at: OffsetDateTime::now_utc(),
    };
    let payload = serde_json::to_value(&lifecycle)?;
    writer
        .insert_order_idempotency(
            &IdempotencyKey::new("live-rate-key")?,
            "live-rate-hash",
            &payload,
        )
        .await?;

    let rebuilt = writer.rebuild_live_order_pending_from_idempotency().await?;
    assert_eq!(rebuilt, 1);
    assert_eq!(writer.pending_live_orders().await?.len(), 1);
    let counts = writer.live_rate_counts(&account, Some(300)).await?;
    assert_eq!(counts.submitted_in_window, 1);
    assert_eq!(counts.submitted_in_session, 1);

    let terminal_lifecycle = LiveOrderLifecycleRecord {
        status: LiveOrderLifecycleStatus::Cancelled,
        updated_at: OffsetDateTime::now_utc(),
        ..lifecycle
    };
    writer
        .insert_order_idempotency(
            &IdempotencyKey::new("live-cancel-key")?,
            "live-cancel-hash",
            &serde_json::to_value(&terminal_lifecycle)?,
        )
        .await?;

    let rebuilt = writer.rebuild_live_order_pending_from_idempotency().await?;
    assert_eq!(rebuilt, 0);
    assert!(writer.pending_live_orders().await?.is_empty());
    let counts = writer.live_rate_counts(&account, Some(300)).await?;
    assert_eq!(counts.submitted_in_window, 1);
    assert_eq!(counts.submitted_in_session, 1);
    Ok(())
}

#[tokio::test]
async fn live_workflow_completion_consumes_approval_and_backlog_atomically()
-> Result<(), Box<dyn std::error::Error>> {
    let key = Arc::new(AuditHmacKey::ephemeral()?);
    let writer = SqliteAuditWriter::connect("sqlite::memory:", key).await?;
    let account = AccountId::from_static("U1234567");
    let approval = ApprovalRecord {
        approval_id: ApprovalId::new(),
        preview_id: OrderPreviewId::new(),
        account_id: account.clone(),
        approved_by: LocalUserId::from_static("operator"),
        status: ApprovalStatus::Approved,
        approved_at: Some(OffsetDateTime::now_utc()),
        expires_at: OffsetDateTime::now_utc() + time::Duration::minutes(5),
    };
    writer.append_approval(&approval).await?;
    let lifecycle = LiveOrderLifecycleRecord {
        account_id: account,
        broker_order_id: BrokerOrderId::from_static("live-atomic"),
        status: LiveOrderLifecycleStatus::Submitted,
        notional: None,
        execution_correlation: None,
        updated_at: OffsetDateTime::now_utc(),
    };
    let payload = serde_json::to_value(&lifecycle)?;
    let idempotency_key = IdempotencyKey::new("live-atomic-key")?;
    writer
        .complete_live_order_workflow(
            &idempotency_key,
            "live-atomic-hash",
            &payload,
            &lifecycle,
            std::slice::from_ref(&approval),
        )
        .await?;

    let replayed = writer
        .replay_order_idempotency(&idempotency_key, "live-atomic-hash")
        .await?
        .ok_or("completed workflow must replay")?;
    assert_eq!(replayed, payload);
    let loaded = writer
        .load_approval(&approval.approval_id)
        .await?
        .ok_or("approval should remain persisted")?;
    assert_eq!(loaded.status, ApprovalStatus::Consumed);
    assert_eq!(writer.pending_live_orders().await?.len(), 1);
    Ok(())
}

#[tokio::test]
async fn concurrent_live_workflow_completion_keeps_transaction_connection()
-> Result<(), Box<dyn std::error::Error>> {
    let key = Arc::new(AuditHmacKey::ephemeral()?);
    let writer = SqliteAuditWriter::connect("sqlite::memory:", key).await?;
    let account = AccountId::from_static("U1234567");
    let approval_a = approval_record(account.clone());
    let approval_b = approval_record(account.clone());
    writer.append_approval(&approval_a).await?;
    writer.append_approval(&approval_b).await?;

    let lifecycle_a = live_lifecycle(account.clone(), "live-concurrent-a");
    let lifecycle_b = live_lifecycle(account, "live-concurrent-b");
    let payload_a = serde_json::to_value(&lifecycle_a)?;
    let payload_b = serde_json::to_value(&lifecycle_b)?;
    let writer_a = writer.clone();
    let writer_b = writer.clone();
    let idempotency_a = IdempotencyKey::new("live-concurrent-key-a")?;
    let idempotency_b = IdempotencyKey::new("live-concurrent-key-b")?;

    let (result_a, result_b) = tokio::join!(
        writer_a.complete_live_order_workflow(
            &idempotency_a,
            "live-concurrent-hash-a",
            &payload_a,
            &lifecycle_a,
            std::slice::from_ref(&approval_a),
        ),
        writer_b.complete_live_order_workflow(
            &idempotency_b,
            "live-concurrent-hash-b",
            &payload_b,
            &lifecycle_b,
            std::slice::from_ref(&approval_b),
        )
    );
    result_a?;
    result_b?;

    assert_eq!(
        writer
            .load_approval(&approval_a.approval_id)
            .await?
            .ok_or("approval a should remain persisted")?
            .status,
        ApprovalStatus::Consumed
    );
    assert_eq!(
        writer
            .load_approval(&approval_b.approval_id)
            .await?
            .ok_or("approval b should remain persisted")?
            .status,
        ApprovalStatus::Consumed
    );
    assert_eq!(writer.pending_live_orders().await?.len(), 2);
    Ok(())
}

fn approval_record(account_id: AccountId) -> ApprovalRecord {
    ApprovalRecord {
        approval_id: ApprovalId::new(),
        preview_id: OrderPreviewId::new(),
        account_id,
        approved_by: LocalUserId::from_static("operator"),
        status: ApprovalStatus::Approved,
        approved_at: Some(OffsetDateTime::now_utc()),
        expires_at: OffsetDateTime::now_utc() + time::Duration::minutes(5),
    }
}

fn live_lifecycle(
    account_id: AccountId,
    broker_order_id: &'static str,
) -> LiveOrderLifecycleRecord {
    LiveOrderLifecycleRecord {
        account_id,
        broker_order_id: BrokerOrderId::from_static(broker_order_id),
        status: LiveOrderLifecycleStatus::Submitted,
        notional: None,
        execution_correlation: None,
        updated_at: OffsetDateTime::now_utc(),
    }
}
