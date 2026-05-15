use ibkr_audit::{AuditEventType, AuditResultStatus};
use ibkr_sidecar::{build_forwarded_broker_request, build_sidecar_relay_audit_event};

#[test]
fn sidecar_audit_correlates_remote_and_local_without_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let forwarded = build_forwarded_broker_request(
        "ibkr_accounts_list",
        "ibkr:accounts:read",
        &serde_json::json!({ "account_hint": "paper" }),
    )?;
    let event =
        build_sidecar_relay_audit_event(&forwarded, "remote-1", AuditResultStatus::Completed, None);

    assert_eq!(event.event_type, AuditEventType::SidecarRelayForwarded);
    assert_eq!(event.request_id, forwarded.request_id);
    assert_eq!(
        event.input_hash.as_deref(),
        Some(forwarded.payload_hash.as_str())
    );
    assert_eq!(event.metadata["remote_instance_id"], "remote-1");
    let serialized = serde_json::to_string(&event)?;
    assert!(!serialized.contains("account_hint"));
    assert!(!serialized.to_ascii_lowercase().contains("cookie"));
    Ok(())
}
