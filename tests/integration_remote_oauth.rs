#[path = "common/remote_oauth.rs"]
mod remote_oauth;

use ibkr_agent_gateway::testing::auth::ACCOUNTS_READ;
use ibkr_agent_gateway::testing::domain::ErrorCode;
use ibkr_agent_gateway::testing::oauth::validate_bearer_jwt;
use time::{Duration, OffsetDateTime};

#[test]
fn validates_valid_remote_oauth_token() -> Result<(), Box<dyn std::error::Error>> {
    let token = remote_oauth::token(
        remote_oauth::ISSUER,
        remote_oauth::AUDIENCE,
        ACCOUNTS_READ,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )?;
    let validated = validate_bearer_jwt(
        &token,
        &remote_oauth::oauth_config(),
        &remote_oauth::jwks(),
        Some(ACCOUNTS_READ),
        OffsetDateTime::now_utc(),
    )?;

    assert_eq!(validated.claims.sub, "user-123");
    assert_eq!(validated.audience, remote_oauth::AUDIENCE);
    assert!(validated.granted_scopes.contains(ACCOUNTS_READ));
    assert!(validated.token_id_hash.is_some());
    Ok(())
}

#[test]
fn validates_rs256_remote_oauth_token() -> Result<(), Box<dyn std::error::Error>> {
    let validated = validate_bearer_jwt(
        remote_oauth::rs256_token(),
        &remote_oauth::oauth_config(),
        &remote_oauth::rsa_jwks(),
        Some(ACCOUNTS_READ),
        OffsetDateTime::now_utc(),
    )?;

    assert_eq!(validated.claims.sub, "user-123");
    assert_eq!(validated.audience, remote_oauth::AUDIENCE);
    assert!(validated.granted_scopes.contains(ACCOUNTS_READ));
    assert!(validated.token_id_hash.is_some());
    Ok(())
}

#[test]
fn rejects_rs256_token_when_jwks_key_does_not_match() -> Result<(), Box<dyn std::error::Error>> {
    let mut jwks = remote_oauth::rsa_jwks();
    jwks.keys[0].kid = Some("another-key".to_string());

    let error = validate_bearer_jwt(
        remote_oauth::rs256_token(),
        &remote_oauth::oauth_config(),
        &jwks,
        Some(ACCOUNTS_READ),
        OffsetDateTime::now_utc(),
    );
    let Err(error) = error else {
        return Err("token should be rejected when JWKS kid does not match".into());
    };
    assert_eq!(error.code, ErrorCode::AuthTokenInvalid);
    Ok(())
}

#[test]
fn token_id_hash_uses_configured_secret() -> Result<(), Box<dyn std::error::Error>> {
    let token = remote_oauth::token(
        remote_oauth::ISSUER,
        remote_oauth::AUDIENCE,
        ACCOUNTS_READ,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )?;
    let mut first_config = remote_oauth::oauth_config();
    first_config.token_id_hmac_secret = b"first-secret".to_vec();
    let mut second_config = remote_oauth::oauth_config();
    second_config.token_id_hmac_secret = b"second-secret".to_vec();

    let first = validate_bearer_jwt(
        &token,
        &first_config,
        &remote_oauth::jwks(),
        Some(ACCOUNTS_READ),
        OffsetDateTime::now_utc(),
    )?;
    let second = validate_bearer_jwt(
        &token,
        &second_config,
        &remote_oauth::jwks(),
        Some(ACCOUNTS_READ),
        OffsetDateTime::now_utc(),
    )?;

    assert_ne!(first.token_id_hash, second.token_id_hash);
    Ok(())
}

#[test]
fn rejects_wrong_issuer_audience_expiry_and_scope() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            remote_oauth::token(
                "https://wrong.example.com/",
                remote_oauth::AUDIENCE,
                ACCOUNTS_READ,
                OffsetDateTime::now_utc() + Duration::minutes(5),
            )?,
            ErrorCode::AuthInvalidIssuer,
        ),
        (
            remote_oauth::token(
                remote_oauth::ISSUER,
                "https://wrong.example.com/mcp",
                ACCOUNTS_READ,
                OffsetDateTime::now_utc() + Duration::minutes(5),
            )?,
            ErrorCode::AuthInvalidAudience,
        ),
        (
            remote_oauth::token(
                remote_oauth::ISSUER,
                remote_oauth::AUDIENCE,
                ACCOUNTS_READ,
                OffsetDateTime::now_utc() - Duration::minutes(5),
            )?,
            ErrorCode::AuthTokenExpired,
        ),
        (
            remote_oauth::token(
                remote_oauth::ISSUER,
                remote_oauth::AUDIENCE,
                "ibkr:health:read",
                OffsetDateTime::now_utc() + Duration::minutes(5),
            )?,
            ErrorCode::AuthMissingScope,
        ),
    ];

    for (token, expected) in cases {
        let error = validate_bearer_jwt(
            &token,
            &remote_oauth::oauth_config(),
            &remote_oauth::jwks(),
            Some(ACCOUNTS_READ),
            OffsetDateTime::now_utc(),
        );
        let Err(error) = error else {
            return Err("token should be rejected".into());
        };
        assert_eq!(error.code, expected);
    }

    Ok(())
}
