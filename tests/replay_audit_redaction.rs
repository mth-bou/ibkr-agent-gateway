use ibkr_audit::{hmac_sha256_hex, is_sensitive_field_name, sha256_hex};

#[test]
fn account_hash_uses_hmac_not_raw_sha256() -> Result<(), Box<dyn std::error::Error>> {
    let account = b"DU1234567";
    let hmac = hmac_sha256_hex(b"local-audit-secret", account)?;
    let raw = sha256_hex(account);

    assert_ne!(hmac, raw);
    assert_eq!(hmac.len(), 64);
    assert!(hmac.chars().all(|character| character.is_ascii_hexdigit()));
    Ok(())
}

#[test]
fn redaction_replay_flags_secret_like_fields() {
    assert!(is_sensitive_field_name("authorization"));
    assert!(is_sensitive_field_name("session_cookie"));
    assert!(is_sensitive_field_name("local_secret_path"));
    assert!(!is_sensitive_field_name("account_mode"));
}
