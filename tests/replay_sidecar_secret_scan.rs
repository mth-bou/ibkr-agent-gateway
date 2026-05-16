use ibkr_agent_gateway::testing::domain::ErrorCode;
use ibkr_agent_gateway::testing::sidecar::build_forwarded_broker_request;

#[test]
fn sidecar_forwarder_rejects_secret_like_payload_fields() {
    let forbidden_payloads = [
        serde_json::json!({ "authorization": "Bearer raw-token" }),
        serde_json::json!({ "cookie": "session=secret" }),
        serde_json::json!({ "local_path": "/home/user/.ibkr/session" }),
        serde_json::json!({ "nested": { "client_secret": "secret" } }),
    ];

    for payload in forbidden_payloads {
        let error =
            build_forwarded_broker_request("ibkr_accounts_list", "ibkr:accounts:read", &payload)
                .expect_err("secret-like payload must be refused");
        assert_eq!(error.code, ErrorCode::OutputUnsafe);
    }
}

#[test]
fn sidecar_forwarded_request_serialization_contains_only_hashes()
-> Result<(), Box<dyn std::error::Error>> {
    let forwarded = build_forwarded_broker_request(
        "ibkr_accounts_list",
        "ibkr:accounts:read",
        &serde_json::json!({ "account_hint": "paper" }),
    )?;
    let serialized = serde_json::to_string(&forwarded)?;

    assert!(serialized.contains("payload_hash"));
    assert!(!serialized.to_ascii_lowercase().contains("cookie"));
    assert!(!serialized.to_ascii_lowercase().contains("authorization"));
    assert!(!serialized.contains("/home/"));
    Ok(())
}
