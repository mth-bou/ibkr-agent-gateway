use ibkr_audit::{AuditResultStatus, RedactionRecord};
use ibkr_domain::{AccountIdHash, ErrorCode};
use ibkr_oauth::audit::build_remote_auth_audit_event;

#[test]
fn remote_oauth_audit_does_not_store_raw_token() -> Result<(), Box<dyn std::error::Error>> {
    let raw_token = "raw-token-that-must-not-appear";
    let event = build_remote_auth_audit_event(
        "user-123",
        "https://issuer.example.com/",
        vec!["ibkr:accounts:read".to_string()],
        AuditResultStatus::DeniedAuth,
        Some(ErrorCode::AuthTokenInvalid),
        AccountIdHash::new("hashed-token-id"),
    );

    assert!(event.redactions.contains(&RedactionRecord {
        field_path: "headers.authorization".to_string(),
        reason: "bearer_token".to_string(),
    }));
    let serialized = serde_json::to_string(&event)?;
    assert!(!serialized.contains(raw_token));
    assert!(
        !serialized
            .to_ascii_lowercase()
            .contains("authorization: bearer")
    );
    assert!(serialized.contains("hashed-token-id"));
    Ok(())
}
