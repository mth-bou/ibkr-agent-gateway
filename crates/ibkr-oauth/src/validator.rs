//! Bearer JWT validation for remote MCP.

use crate::jwks::Jwks;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use ibkr_domain::{AccountIdHash, ErrorCode, GatewayError};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeSet;
use time::OffsetDateTime;

type HmacSha256 = Hmac<Sha256>;

/// OAuth issuer validation config.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OAuthIssuerConfig {
    /// Expected issuer.
    pub issuer: String,
    /// JWKS URL recorded for metadata/audit.
    pub jwks_url: String,
    /// Accepted audiences/resources.
    pub audiences: Vec<String>,
    /// Scopes that this gateway may grant remotely.
    pub allowed_scopes: Vec<String>,
    /// Clock skew in seconds.
    pub clock_skew_seconds: u64,
    /// Optional authorization server metadata URL.
    pub metadata_url: Option<String>,
    /// HMAC secret used only to hash token ids for audit.
    pub token_id_hmac_secret: Vec<u8>,
}

/// JWT claims used by remote MCP authorization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OAuthTokenClaims {
    /// Subject.
    pub sub: String,
    /// Issuer.
    pub iss: String,
    /// Audience.
    pub aud: TokenAudience,
    /// Expiry as Unix timestamp.
    pub exp: i64,
    /// Not-before as Unix timestamp.
    pub nbf: Option<i64>,
    /// Issued-at as Unix timestamp.
    pub iat: Option<i64>,
    /// Space-delimited OAuth scopes.
    pub scope: Option<String>,
    /// Token id.
    pub jti: Option<String>,
    /// Optional tenant id.
    pub tenant_id: Option<String>,
}

/// JWT audience can be a string or an array.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TokenAudience {
    /// Single audience.
    Single(String),
    /// Multiple audiences.
    Multiple(Vec<String>),
}

impl TokenAudience {
    fn values(&self) -> Vec<&str> {
        match self {
            Self::Single(value) => vec![value.as_str()],
            Self::Multiple(values) => values.iter().map(String::as_str).collect(),
        }
    }
}

/// Validated remote token details.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedOAuthToken {
    /// Claims.
    pub claims: OAuthTokenClaims,
    /// Matched audience.
    pub audience: String,
    /// Granted gateway scopes after intersecting with allowed scopes.
    pub granted_scopes: BTreeSet<String>,
    /// HMAC hash of `jti` when present.
    pub token_id_hash: Option<AccountIdHash>,
}

#[derive(Deserialize)]
struct JwtHeader {
    alg: String,
    kid: Option<String>,
}

/// Validates a bearer JWT and required gateway scope.
pub fn validate_bearer_jwt(
    token: &str,
    config: &OAuthIssuerConfig,
    jwks: &Jwks,
    required_scope: Option<&str>,
    now: OffsetDateTime,
) -> Result<ValidatedOAuthToken, GatewayError> {
    let (header, claims, signing_input, signature) = decode_parts(token)?;
    verify_signature(&header, jwks, signing_input, &signature)?;
    validate_claims(&claims, config, required_scope, now)
}

fn decode_parts(
    token: &str,
) -> Result<(JwtHeader, OAuthTokenClaims, String, Vec<u8>), GatewayError> {
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(invalid_token(
            "JWT must contain header, claims, and signature",
        ));
    }

    let header_bytes = URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|_| invalid_token("JWT header is not valid base64url"))?;
    let claims_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| invalid_token("JWT claims are not valid base64url"))?;
    let signature = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| invalid_token("JWT signature is not valid base64url"))?;
    let header = serde_json::from_slice::<JwtHeader>(&header_bytes)
        .map_err(|_| invalid_token("JWT header is not valid JSON"))?;
    let claims = serde_json::from_slice::<OAuthTokenClaims>(&claims_bytes)
        .map_err(|_| invalid_token("JWT claims are not valid JSON"))?;

    Ok((
        header,
        claims,
        format!("{}.{}", parts[0], parts[1]),
        signature,
    ))
}

fn verify_signature(
    header: &JwtHeader,
    jwks: &Jwks,
    signing_input: String,
    signature: &[u8],
) -> Result<(), GatewayError> {
    if header.alg != "HS256" {
        return Err(invalid_token(
            "Only HS256 JWKS validation is supported in this phase",
        ));
    }

    let key = jwks
        .select_key(header.kid.as_deref())
        .ok_or_else(|| invalid_token("JWT key id is not present in JWKS"))?;
    if key.kty != "oct" {
        return Err(invalid_token("JWKS key type is not supported"));
    }
    if key.alg.as_deref().is_some_and(|alg| alg != "HS256") {
        return Err(invalid_token("JWKS key algorithm does not match JWT"));
    }
    let Some(k) = &key.k else {
        return Err(invalid_token("JWKS symmetric key material is missing"));
    };
    let secret = URL_SAFE_NO_PAD
        .decode(k)
        .map_err(|_| invalid_token("JWKS key material is not valid base64url"))?;
    let mut mac = HmacSha256::new_from_slice(&secret)
        .map_err(|_| invalid_token("JWKS key material is invalid"))?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(signature)
        .map_err(|_| invalid_token("JWT signature is invalid"))
}

fn validate_claims(
    claims: &OAuthTokenClaims,
    config: &OAuthIssuerConfig,
    required_scope: Option<&str>,
    now: OffsetDateTime,
) -> Result<ValidatedOAuthToken, GatewayError> {
    if claims.iss != config.issuer {
        return Err(GatewayError::new(
            ErrorCode::AuthInvalidIssuer,
            "JWT issuer is not allowed",
            false,
            Some("Use a token from the configured issuer".to_string()),
        ));
    }

    let audience = claims
        .aud
        .values()
        .into_iter()
        .find(|audience| config.audiences.iter().any(|allowed| allowed == audience))
        .ok_or_else(|| {
            GatewayError::new(
                ErrorCode::AuthInvalidAudience,
                "JWT audience/resource is not allowed",
                false,
                Some("Request a token for this gateway resource".to_string()),
            )
        })?
        .to_string();

    let skew = i64::try_from(config.clock_skew_seconds).unwrap_or(i64::MAX);
    let now_unix = now.unix_timestamp();
    if claims.exp.saturating_add(skew) < now_unix {
        return Err(GatewayError::new(
            ErrorCode::AuthTokenExpired,
            "JWT is expired",
            false,
            Some("Refresh the MCP access token".to_string()),
        ));
    }
    if claims
        .nbf
        .is_some_and(|not_before| not_before.saturating_sub(skew) > now_unix)
    {
        return Err(invalid_token("JWT is not valid yet"));
    }

    let allowed = config
        .allowed_scopes
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let granted_scopes = claims
        .scope
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .filter(|scope| allowed.contains(scope))
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();

    if let Some(required_scope) = required_scope {
        if !granted_scopes.contains(required_scope) {
            return Err(GatewayError::new(
                ErrorCode::AuthMissingScope,
                format!("Missing required scope: {required_scope}"),
                false,
                Some("Request a token with the required gateway scope".to_string()),
            ));
        }
    }

    let token_id_hash = claims
        .jti
        .as_deref()
        .map(|jti| hmac_identifier(&config.token_id_hmac_secret, jti))
        .transpose()?
        .map(AccountIdHash::from_hash);

    Ok(ValidatedOAuthToken {
        claims: claims.clone(),
        audience,
        granted_scopes,
        token_id_hash,
    })
}

fn hmac_identifier(secret: &[u8], value: &str) -> Result<String, GatewayError> {
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| invalid_token("Token hash key is invalid"))?;
    mac.update(value.as_bytes());
    let bytes = mac.finalize().into_bytes();
    Ok(bytes_to_lower_hex(&bytes))
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn invalid_token(message: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::AuthTokenInvalid,
        message,
        false,
        Some("Provide a valid bearer token".to_string()),
    )
}
