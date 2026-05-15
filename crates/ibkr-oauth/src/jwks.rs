//! JWKS and authorization-server metadata discovery.

use ibkr_domain::{ErrorCode, GatewayError};
use serde::{Deserialize, Serialize};
use url::Url;

/// One JSON Web Key.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Jwk {
    /// Key id.
    pub kid: Option<String>,
    /// Key type. This phase supports `oct` keys for deterministic validation.
    pub kty: String,
    /// Algorithm.
    pub alg: Option<String>,
    /// Base64url-encoded symmetric key material for `oct` keys.
    pub k: Option<String>,
}

/// JSON Web Key Set.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Jwks {
    /// Keys.
    pub keys: Vec<Jwk>,
}

impl Jwks {
    /// Selects a key by `kid`, or the only key when no `kid` is present.
    #[must_use]
    pub fn select_key(&self, kid: Option<&str>) -> Option<&Jwk> {
        match kid {
            Some(kid) => self.keys.iter().find(|key| key.kid.as_deref() == Some(kid)),
            None if self.keys.len() == 1 => self.keys.first(),
            None => None,
        }
    }
}

/// Fetches JWKS from an endpoint.
pub async fn fetch_jwks(url: &Url) -> Result<Jwks, GatewayError> {
    let response = reqwest::get(url.clone()).await.map_err(|_| {
        GatewayError::new(
            ErrorCode::AuthTokenInvalid,
            "Unable to fetch JWKS",
            true,
            Some("Verify the configured JWKS URL".to_string()),
        )
    })?;

    response.json::<Jwks>().await.map_err(|_| {
        GatewayError::new(
            ErrorCode::AuthTokenInvalid,
            "Unable to parse JWKS",
            true,
            Some("Verify the authorization server JWKS response".to_string()),
        )
    })
}
