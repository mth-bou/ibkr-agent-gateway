//! JWKS and authorization-server metadata discovery.

use crate::internal::domain::{ErrorCode, GatewayError};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration as StdDuration};
use time::{Duration, OffsetDateTime};
use url::Url;

const DEFAULT_TIMEOUT: StdDuration = StdDuration::from_secs(5);
const DEFAULT_MAX_BODY_BYTES: usize = 64 * 1024;
const DEFAULT_CACHE_TTL: Duration = Duration::minutes(10);

/// One JSON Web Key.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Jwk {
    /// Key id.
    pub kid: Option<String>,
    /// Key type. Production OIDC uses `RSA`; `oct` remains for deterministic local tests.
    pub kty: String,
    /// Algorithm.
    pub alg: Option<String>,
    /// Base64url-encoded symmetric key material for `oct` keys.
    pub k: Option<String>,
    /// Base64url-encoded RSA modulus for `RSA` keys.
    pub n: Option<String>,
    /// Base64url-encoded RSA public exponent for `RSA` keys.
    pub e: Option<String>,
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

/// HTTP client for bounded JWKS discovery.
#[derive(Clone, Debug)]
pub struct JwksHttpClient {
    http: reqwest::Client,
    max_body_bytes: usize,
}

impl JwksHttpClient {
    /// Builds a JWKS client with request timeout and maximum response size.
    ///
    /// Redirects are disabled to prevent a malicious or misconfigured JWKS host
    /// from pivoting requests to internal endpoints (e.g. cloud metadata at
    /// `169.254.169.254`).
    pub fn new(timeout: StdDuration, max_body_bytes: usize) -> Result<Self, GatewayError> {
        let http = reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| jwks_error("Unable to initialize JWKS HTTP client"))?;

        Ok(Self {
            http,
            max_body_bytes,
        })
    }

    /// Fetches and parses a JWKS document with status and size checks.
    ///
    /// Rejects non-HTTPS URLs before issuing the request as defence in depth
    /// against misconfigured discovery endpoints.
    pub async fn fetch(&self, url: &Url) -> Result<Jwks, GatewayError> {
        if url.scheme() != "https" {
            return Err(jwks_error("JWKS URL must use the https scheme"));
        }
        let response = self
            .http
            .get(url.clone())
            .send()
            .await
            .map_err(|_| jwks_error("Unable to fetch JWKS"))?
            .error_for_status()
            .map_err(|_| jwks_error("JWKS endpoint returned an unsuccessful status"))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|_| jwks_error("Unable to read JWKS response body"))?;
        if bytes.len() > self.max_body_bytes {
            return Err(jwks_error("JWKS response body is too large"));
        }

        serde_json::from_slice::<Jwks>(&bytes).map_err(|_| jwks_error("Unable to parse JWKS"))
    }
}

impl Default for JwksHttpClient {
    fn default() -> Self {
        Self::new(DEFAULT_TIMEOUT, DEFAULT_MAX_BODY_BYTES).unwrap_or_else(|_| Self {
            http: reqwest::Client::new(),
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        })
    }
}

#[derive(Clone, Debug)]
struct CachedJwks {
    jwks: Jwks,
    fetched_at: OffsetDateTime,
}

/// In-memory JWKS cache keyed by endpoint URL.
#[derive(Clone, Debug)]
pub struct JwksCache {
    ttl: Duration,
    entries: BTreeMap<String, CachedJwks>,
}

impl JwksCache {
    /// Creates a cache with the provided TTL.
    #[must_use]
    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: BTreeMap::new(),
        }
    }

    /// Stores a JWKS document at a fetch time.
    pub fn insert(&mut self, url: &Url, jwks: Jwks, now: OffsetDateTime) {
        self.entries.insert(
            url.to_string(),
            CachedJwks {
                jwks,
                fetched_at: now,
            },
        );
    }

    /// Returns a cached JWKS document when it is still within TTL.
    #[must_use]
    pub fn get_cached(&self, url: &Url, now: OffsetDateTime) -> Option<&Jwks> {
        let cached = self.entries.get(url.as_str())?;
        if now - cached.fetched_at <= self.ttl {
            Some(&cached.jwks)
        } else {
            None
        }
    }

    /// Returns a cached JWKS document or fetches and stores a fresh one.
    pub async fn get_or_fetch(
        &mut self,
        client: &JwksHttpClient,
        url: &Url,
        now: OffsetDateTime,
    ) -> Result<Jwks, GatewayError> {
        if let Some(jwks) = self.get_cached(url, now) {
            return Ok(jwks.clone());
        }

        let jwks = client.fetch(url).await?;
        self.insert(url, jwks.clone(), now);
        Ok(jwks)
    }
}

impl Default for JwksCache {
    fn default() -> Self {
        Self::with_ttl(DEFAULT_CACHE_TTL)
    }
}

/// Fetches JWKS from an endpoint.
pub async fn fetch_jwks(url: &Url) -> Result<Jwks, GatewayError> {
    JwksHttpClient::default().fetch(url).await
}

fn jwks_error(message: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::AuthTokenInvalid,
        message,
        true,
        Some("Verify the configured JWKS URL and authorization server response".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::JwksHttpClient;
    use crate::internal::domain::ErrorCode;
    use std::time::Duration as StdDuration;
    use url::Url;

    #[tokio::test]
    async fn rejects_non_https_jwks_url() {
        let Ok(client) = JwksHttpClient::new(StdDuration::from_secs(1), 1024) else {
            unreachable!("client should build");
        };
        let Ok(url) = Url::parse("http://issuer.example.com/jwks.json") else {
            unreachable!("static URL should parse");
        };
        let Err(error) = client.fetch(&url).await else {
            unreachable!("http scheme must be rejected without a network call");
        };
        assert_eq!(error.code, ErrorCode::AuthTokenInvalid);
    }

    #[tokio::test]
    async fn rejects_file_scheme_jwks_url() {
        let Ok(client) = JwksHttpClient::new(StdDuration::from_secs(1), 1024) else {
            unreachable!("client should build");
        };
        let Ok(url) = Url::parse("file:///etc/passwd") else {
            unreachable!("static URL should parse");
        };
        let Err(error) = client.fetch(&url).await else {
            unreachable!("file scheme must be rejected");
        };
        assert_eq!(error.code, ErrorCode::AuthTokenInvalid);
    }
}
