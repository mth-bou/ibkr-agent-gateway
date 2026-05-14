use ibkr_audit::{AuditDecision, AuditEvent, AuditEventType, AuditResultStatus, SqliteAuditWriter};
use ibkr_domain::{AuditEventId, LocalUserId, RequestId, SessionId};
use std::collections::BTreeMap;
use time::OffsetDateTime;

#[tokio::test]
async fn appends_audit_event_to_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    let writer = SqliteAuditWriter::connect("sqlite::memory:").await?;
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
