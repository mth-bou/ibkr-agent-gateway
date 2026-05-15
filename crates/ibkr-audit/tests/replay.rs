use ibkr_audit::{
    AuditDecision, AuditEvent, AuditEventType, AuditResultStatus, AuditTail, AuditTailRecord,
    RedactionRecord, ReplayCase, SecretScanExpectation, export_audit_tail_jsonl, replay_case,
};
use ibkr_domain::{AuditEventId, LocalUserId, RequestId, SessionId};
use std::collections::BTreeMap;
use time::OffsetDateTime;

#[test]
fn replay_fixture_reproduces_expected_decision() -> Result<(), Box<dyn std::error::Error>> {
    let case = ReplayCase {
        case_id: "readonly-health-allow".to_string(),
        input_event: audit_event(AuditDecision::Allow),
        expected_decision: AuditDecision::Allow,
        expected_output_shape: "decision::allow".to_string(),
        secret_scan_expectation: SecretScanExpectation::Pass,
    };

    let outcome = replay_case(&case)?;

    assert!(outcome.matched);
    assert_eq!(outcome.output_shape, "decision::allow");
    Ok(())
}

#[test]
fn audit_export_is_jsonl_and_hashed() -> Result<(), Box<dyn std::error::Error>> {
    let tail = AuditTail {
        events: vec![AuditTailRecord {
            sequence_id: 1,
            event: audit_event(AuditDecision::Allow),
        }],
    };

    let export = export_audit_tail_jsonl(&tail, 1)?;

    assert_eq!(export.range.exported_count, 1);
    assert_eq!(export.file_hash.len(), 64);
    assert!(export.payload_jsonl.contains("\"sequence_id\":1"));
    assert!(
        !export
            .payload_jsonl
            .to_ascii_lowercase()
            .contains("bearer ")
    );
    Ok(())
}

fn audit_event(decision: AuditDecision) -> AuditEvent {
    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: AuditEventType::ToolCompleted,
        timestamp: OffsetDateTime::UNIX_EPOCH,
        user_id: LocalUserId::from_static("local-user"),
        session_id: SessionId::new(),
        request_id: RequestId::new(),
        account_id_hash: None,
        tool_name: Some("ibkr_health".to_string()),
        scopes: vec!["ibkr:health:read".to_string()],
        decision,
        result_status: AuditResultStatus::Completed,
        error_code: None,
        input_hash: Some("input-hash".to_string()),
        output_hash: Some("output-hash".to_string()),
        redactions: vec![RedactionRecord {
            field_path: "headers.authorization".to_string(),
            reason: "redacted".to_string(),
        }],
        metadata: BTreeMap::new(),
    }
}
