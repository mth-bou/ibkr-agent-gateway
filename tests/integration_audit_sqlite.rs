use ibkr_agent_gateway::testing::audit::{
    AuditDecision, AuditEvent, AuditEventType, AuditHmacKey, AuditResultStatus, SqliteAuditWriter,
};
use ibkr_agent_gateway::testing::domain::{AuditEventId, LocalUserId, RequestId, SessionId};
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
    use ibkr_agent_gateway::testing::domain::ErrorCode;
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
