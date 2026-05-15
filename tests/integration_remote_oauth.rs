#[path = "common/remote_oauth.rs"]
mod remote_oauth;

use ibkr_auth::ACCOUNTS_READ;
use ibkr_domain::ErrorCode;
use ibkr_oauth::validate_bearer_jwt;
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
