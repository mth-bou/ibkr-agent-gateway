#![allow(dead_code)]

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use ibkr_auth::{ACCOUNTS_READ, HEALTH_READ};
use ibkr_config::RemoteMcpConfig;
use ibkr_oauth::{Jwk, Jwks, OAuthIssuerConfig};
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
        }],
    }
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
