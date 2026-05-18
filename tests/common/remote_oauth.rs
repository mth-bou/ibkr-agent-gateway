#![allow(dead_code)]

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use ibkr_agent_gateway::testing::auth::{ACCOUNTS_READ, HEALTH_READ};
use ibkr_agent_gateway::testing::config::RemoteMcpConfig;
use ibkr_agent_gateway::testing::oauth::{Jwk, Jwks, OAuthIssuerConfig};
use serde_json::json;
use sha2::Sha256;
use time::OffsetDateTime;
use url::Url;

type HmacSha256 = Hmac<Sha256>;

pub const SECRET: &[u8] = b"remote-oauth-test-secret";
pub const ISSUER: &str = "https://issuer.example.com/";
pub const AUDIENCE: &str = "https://gateway.example.com/mcp";

pub fn remote_config() -> Result<RemoteMcpConfig, Box<dyn std::error::Error>> {
    Ok(RemoteMcpConfig {
        enabled: true,
        bind_address: "127.0.0.1:8080".to_string(),
        resource: Some(Url::parse(AUDIENCE)?),
        issuer: Some(Url::parse(ISSUER)?),
        jwks_url: Some(Url::parse("https://issuer.example.com/jwks.json")?),
        metadata_url: Some(Url::parse(
            "https://issuer.example.com/.well-known/openid-configuration",
        )?),
        audiences: vec![AUDIENCE.to_string()],
        allowed_scopes: vec![HEALTH_READ.to_string(), ACCOUNTS_READ.to_string()],
        clock_skew_seconds: 30,
        rate_limit_max_requests: 120,
        rate_limit_window_seconds: 60,
        max_connections: 64,
        token_id_hmac_secret: Some("token-id-test-secret".to_string()),
    })
}

pub fn oauth_config() -> OAuthIssuerConfig {
    OAuthIssuerConfig {
        issuer: ISSUER.to_string(),
        jwks_url: "https://issuer.example.com/jwks.json".to_string(),
        audiences: vec![AUDIENCE.to_string()],
        allowed_scopes: vec![HEALTH_READ.to_string(), ACCOUNTS_READ.to_string()],
        clock_skew_seconds: 30,
        metadata_url: Some(
            "https://issuer.example.com/.well-known/openid-configuration".to_string(),
        ),
        token_id_hmac_secret: b"token-id-test-secret".to_vec(),
    }
}

pub fn jwks() -> Jwks {
    Jwks {
        keys: vec![Jwk {
            kid: Some("test-key".to_string()),
            kty: "oct".to_string(),
            alg: Some("HS256".to_string()),
            k: Some(URL_SAFE_NO_PAD.encode(SECRET)),
            n: None,
            e: None,
        }],
    }
}

pub fn rsa_jwks() -> Jwks {
    Jwks {
        keys: vec![Jwk {
            kid: Some("rsa-test-key".to_string()),
            kty: "RSA".to_string(),
            alg: Some("RS256".to_string()),
            k: None,
            n: Some(
                "ur-vqqomgXzlcaDFTASqF_nWlqLqubiXhVwaBjsVlU0jlJlcspg4uMHo26Mgi3M8tF2VAZxDfpAVqbjZlcMPJW7gZqAjtD1dgkyCfwqwSdpRmlfdXSXhYwLAV9VsfL8ItWWia-rcA8A3t31FVDWu6B-G4UmGExIjODNkgcNTN_8_9sLvyobtRBNxu3NqJZlUVvWWtRRc5Lrf9Ve4rT_6jtR5qnuabOMfgHPRbmagZGUOO0rfwOt0mFFaSJZ_xpgV5pJ4akXoj-TdOvJoYheyhc5hc2wLtzGEK1YBrrC5hLSxuXWumqsZvbpTlZeW4dqRPdylbNjXkq1qJdMtbhLKgw"
                    .to_string(),
            ),
            e: Some("AQAB".to_string()),
        }],
    }
}

pub fn rs256_token() -> &'static str {
    "eyJhbGciOiJSUzI1NiIsImtpZCI6InJzYS10ZXN0LWtleSJ9.eyJzdWIiOiJ1c2VyLTEyMyIsImlzcyI6Imh0dHBzOi8vaXNzdWVyLmV4YW1wbGUuY29tLyIsImF1ZCI6Imh0dHBzOi8vZ2F0ZXdheS5leGFtcGxlLmNvbS9tY3AiLCJleHAiOjQxMDI0NDQ4MDAsImlhdCI6MTc2MDAwMDAwMCwic2NvcGUiOiJpYmtyOmFjY291bnRzOnJlYWQiLCJqdGkiOiJ0b2tlbi1pZC0xMjMifQ.PKHDHwihOWXSrpoKZGFON449484o73HyjIrqDit7CXlbN3HyBvCXqZg2hRt1DunqBSO_GLgYbgTQUuCS5o0GinoFzhbtJ0fufdGn4zHxdQ9jAdQWWIJ1SAI0kA978_l1mD517ugYO6GZhKt2m64BFt_Q6a618XOUjmPUqgRWXbv6GapsGq2Q0y3G6Nb1T_NwuKqGCrHDe3EiCJgFckQuK1d8SH-skXTckP8qQmgEtp4fb0jm48oVdCQ2l9gYiqIfIE28qxOUag2bgHkvRGygAEAsy30wxIqy3A66TYkSNPXMxDRCE6XHTZHrG1QN90Z0GqwP0de5-jhVLCLNEwUNvw"
}

pub fn token(
    issuer: &str,
    audience: &str,
    scope: &str,
    expires_at: OffsetDateTime,
) -> Result<String, Box<dyn std::error::Error>> {
    let header = json!({
        "alg": "HS256",
        "kid": "test-key"
    });
    let claims = json!({
        "sub": "user-123",
        "iss": issuer,
        "aud": audience,
        "exp": expires_at.unix_timestamp(),
        "iat": OffsetDateTime::now_utc().unix_timestamp(),
        "scope": scope,
        "jti": "token-id-123"
    });
    sign(header, claims)
}

fn sign(
    header: serde_json::Value,
    claims: serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
    let header = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?);
    let claims = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims)?);
    let signing_input = format!("{header}.{claims}");
    let mut mac = HmacSha256::new_from_slice(SECRET)?;
    mac.update(signing_input.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{signing_input}.{signature}"))
}
