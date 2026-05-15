use ibkr_domain::LocalUserId;
use ibkr_mcp::sidecar_relay::accept_sidecar_relay_request;
use ibkr_sidecar::{PairingStatus, SidecarId, create_pairing, create_relay_session};
use time::OffsetDateTime;

#[test]
fn pairing_binds_sidecar_and_remote_session() -> Result<(), Box<dyn std::error::Error>> {
    let sidecar_id = SidecarId::new();
    let pairing = create_pairing(
        "remote-1",
        sidecar_id.clone(),
        LocalUserId::from_static("user-1"),
        300,
    );
    assert_eq!(pairing.status, PairingStatus::Active);
    assert_eq!(pairing.remote_instance_id, "remote-1");
    assert_eq!(pairing.sidecar_id, sidecar_id);

    let session = create_relay_session(pairing.sidecar_id.clone(), "remote-1", 60);
    let accepted = accept_sidecar_relay_request(
        Some(&session),
        "ibkr_accounts_list",
        "ibkr:accounts:read",
        &serde_json::json!({ "account_hint": "paper" }),
        OffsetDateTime::now_utc(),
    )?;

    assert_eq!(accepted.forwarded_request.tool_name, "ibkr_accounts_list");
    assert_eq!(accepted.forwarded_request.scope, "ibkr:accounts:read");
    assert!(!accepted.forwarded_request.payload_hash.is_empty());
    Ok(())
}
