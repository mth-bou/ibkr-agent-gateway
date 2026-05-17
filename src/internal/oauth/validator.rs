//! Bearer JWT validation for remote MCP.
//!
//! Production code paths accept only RS256 signatures over `kty=RSA` JWKS
//! material. Symmetric `kty=oct` keys and `HS256` signatures are compiled in
//! only with the `unstable-internal-test-support` feature so deterministic
//! tests can exercise the validator without an asymmetric signing harness.

use super::jwks::Jwks;
use crate::internal::config::remote_mcp::MAX_CLOCK_SKEW_SECONDS;
use crate::internal::domain::{AccountIdHash, ErrorCode, GatewayError};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use ring::signature;
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
    fn matched_audience<'a>(&'a self, allowed: &BTreeSet<String>) -> Option<&'a str> {
        match self {
            Self::Single(value) if allowed.contains(value) => Some(value.as_str()),
            Self::Single(_) => None,
            Self::Multiple(values) => values
                .iter()
                .find(|value| allowed.contains(*value))
                .map(String::as_str),
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

/// Precompiled OAuth verifier for remote MCP hot paths.
#[derive(Clone, Debug)]
pub struct PreparedOAuthVerifier {
    config: OAuthIssuerConfig,
    allowed_audiences: BTreeSet<String>,
    allowed_scopes: BTreeSet<String>,
    jwks: PreparedJwks,
}

impl PreparedOAuthVerifier {
    /// Prepares issuer config and JWKS key material once for repeated token validation.
    pub fn new(config: OAuthIssuerConfig, jwks: &Jwks) -> Result<Self, GatewayError> {
        let allowed_audiences = config.audiences.iter().cloned().collect();
        let allowed_scopes = config.allowed_scopes.iter().cloned().collect();
        let jwks = PreparedJwks::new(jwks)?;
        Ok(Self {
            config,
            allowed_audiences,
            allowed_scopes,
            jwks,
        })
    }

    /// Validates a bearer JWT and required gateway scope.
    pub fn validate_bearer_jwt(
        &self,
        token: &str,
        required_scope: Option<&str>,
        now: OffsetDateTime,
    ) -> Result<ValidatedOAuthToken, GatewayError> {
        let (header, claims, signing_input, signature) = decode_parts(token)?;
        verify_signature(&header, &self.jwks, signing_input, &signature)?;
        validate_claims(
            &claims,
            &self.config,
            &self.allowed_audiences,
            &self.allowed_scopes,
            required_scope,
            now,
        )
    }
}

#[derive(Clone, Debug)]
struct PreparedJwks {
    keys: Vec<PreparedJwk>,
}

impl PreparedJwks {
    fn new(jwks: &Jwks) -> Result<Self, GatewayError> {
        let mut keys = Vec::with_capacity(jwks.keys.len());
        for key in &jwks.keys {
            let material = match key.kty.as_str() {
                "RSA" => PreparedJwkMaterial::Rsa {
                    n: decode_required_key_material(key.n.as_deref(), "JWKS RSA modulus")?,
                    e: decode_required_key_material(key.e.as_deref(), "JWKS RSA exponent")?,
                },
                #[cfg(feature = "unstable-internal-test-support")]
                "oct" => PreparedJwkMaterial::Hmac {
                    secret: decode_required_key_material(
                        key.k.as_deref(),
                        "JWKS symmetric key material",
                    )?,
                },
                _ => continue,
            };
            keys.push(PreparedJwk {
                kid: key.kid.clone(),
                alg: key.alg.clone(),
                material,
            });
        }

        if keys.is_empty() {
            return Err(invalid_token(
                "JWKS does not contain supported key material",
            ));
        }

        Ok(Self { keys })
    }

    fn select_key(&self, kid: Option<&str>) -> Option<&PreparedJwk> {
        match kid {
            Some(kid) => self.keys.iter().find(|key| key.kid.as_deref() == Some(kid)),
            None if self.keys.len() == 1 => self.keys.first(),
            None => None,
        }
    }
}

#[derive(Clone, Debug)]
struct PreparedJwk {
    kid: Option<String>,
    alg: Option<String>,
    material: PreparedJwkMaterial,
}

#[derive(Clone, Debug)]
enum PreparedJwkMaterial {
    Rsa {
        n: Vec<u8>,
        e: Vec<u8>,
    },
    #[cfg(feature = "unstable-internal-test-support")]
    Hmac {
        secret: Vec<u8>,
    },
}

/// Validates a bearer JWT and required gateway scope.
pub fn validate_bearer_jwt(
    token: &str,
    config: &OAuthIssuerConfig,
    jwks: &Jwks,
    required_scope: Option<&str>,
    now: OffsetDateTime,
) -> Result<ValidatedOAuthToken, GatewayError> {
    PreparedOAuthVerifier::new(config.clone(), jwks)?.validate_bearer_jwt(
        token,
        required_scope,
        now,
    )
}

fn decode_parts(token: &str) -> Result<(JwtHeader, OAuthTokenClaims, &str, Vec<u8>), GatewayError> {
    let Some((header_part, rest)) = token.split_once('.') else {
        return Err(invalid_token(
            "JWT must contain header, claims, and signature",
        ));
    };
    let Some((claims_part, signature_part)) = rest.split_once('.') else {
        return Err(invalid_token(
            "JWT must contain header, claims, and signature",
        ));
    };
    if signature_part.contains('.') {
        return Err(invalid_token(
            "JWT must contain header, claims, and signature",
        ));
    }
    let signing_input_len = header_part.len() + 1 + claims_part.len();
    let signing_input = &token[..signing_input_len];

    let header_bytes = URL_SAFE_NO_PAD
        .decode(header_part)
        .map_err(|_| invalid_token("JWT header is not valid base64url"))?;
    let claims_bytes = URL_SAFE_NO_PAD
        .decode(claims_part)
        .map_err(|_| invalid_token("JWT claims are not valid base64url"))?;
    let signature = URL_SAFE_NO_PAD
        .decode(signature_part)
        .map_err(|_| invalid_token("JWT signature is not valid base64url"))?;
    let header = serde_json::from_slice::<JwtHeader>(&header_bytes)
        .map_err(|_| invalid_token("JWT header is not valid JSON"))?;
    let claims = serde_json::from_slice::<OAuthTokenClaims>(&claims_bytes)
        .map_err(|_| invalid_token("JWT claims are not valid JSON"))?;

    Ok((header, claims, signing_input, signature))
}

fn verify_signature(
    header: &JwtHeader,
    jwks: &PreparedJwks,
    signing_input: &str,
    signature: &[u8],
) -> Result<(), GatewayError> {
    let key = jwks
        .select_key(header.kid.as_deref())
        .ok_or_else(|| invalid_token("JWT key id is not present in JWKS"))?;
    if key.alg.as_deref().is_some_and(|alg| alg != header.alg) {
        return Err(invalid_token("JWKS key algorithm does not match JWT"));
    }

    match (header.alg.as_str(), &key.material) {
        ("RS256", PreparedJwkMaterial::Rsa { n, e }) => {
            verify_rs256(n, e, signing_input, signature)
        }
        #[cfg(feature = "unstable-internal-test-support")]
        ("HS256", PreparedJwkMaterial::Hmac { secret }) => {
            verify_hs256(secret, signing_input, signature)
        }
        #[cfg(feature = "unstable-internal-test-support")]
        ("RS256" | "HS256", _) => Err(invalid_token("JWKS key type does not match JWT algorithm")),
        // In production builds, `PreparedJwkMaterial` only has the `Rsa` variant,
        // so the `("RS256", _)` arm above is provably unreachable. The catch-all
        // below still covers any other JWT `alg` value (including a hostile
        // `HS256` whose key material was filtered out at parse time).
        _ => Err(invalid_token("JWT signature algorithm is not supported")),
    }
}

#[cfg(feature = "unstable-internal-test-support")]
fn verify_hs256(secret: &[u8], signing_input: &str, signature: &[u8]) -> Result<(), GatewayError> {
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| invalid_token("JWKS key material is invalid"))?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(signature)
        .map_err(|_| invalid_token("JWT signature is invalid"))
}

fn verify_rs256(
    n: &[u8],
    e: &[u8],
    signing_input: &str,
    signature: &[u8],
) -> Result<(), GatewayError> {
    let public_key = signature::RsaPublicKeyComponents { n: &n, e: &e };
    public_key
        .verify(
            &signature::RSA_PKCS1_2048_8192_SHA256,
            signing_input.as_bytes(),
            signature,
        )
        .map_err(|_| invalid_token("JWT signature is invalid"))
}

fn validate_claims(
    claims: &OAuthTokenClaims,
    config: &OAuthIssuerConfig,
    allowed_audiences: &BTreeSet<String>,
    allowed_scopes: &BTreeSet<String>,
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
        .matched_audience(allowed_audiences)
        .ok_or_else(|| {
            GatewayError::new(
                ErrorCode::AuthInvalidAudience,
                "JWT audience/resource is not allowed",
                false,
                Some("Request a token for this gateway resource".to_string()),
            )
        })?
        .to_string();

    // Defence in depth: even if a malformed config slipped past validation,
    // clamp the skew here so a `u64::MAX` value cannot disable expiry.
    let clamped_skew = config.clock_skew_seconds.min(MAX_CLOCK_SKEW_SECONDS);
    let skew = i64::try_from(clamped_skew).unwrap_or(i64::MAX);
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

    let granted_scopes = claims
        .scope
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .filter(|scope| allowed_scopes.contains(*scope))
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();

    if let Some(required_scope) = required_scope
        && !granted_scopes.contains(required_scope)
    {
        return Err(GatewayError::new(
            ErrorCode::AuthMissingScope,
            format!("Missing required scope: {required_scope}"),
            false,
            Some("Request a token with the required gateway scope".to_string()),
        ));
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

fn decode_required_key_material(value: Option<&str>, label: &str) -> Result<Vec<u8>, GatewayError> {
    let Some(value) = value else {
        return Err(invalid_token(&format!("{label} is missing")));
    };
    URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| invalid_token(&format!("{label} is not valid base64url")))
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
