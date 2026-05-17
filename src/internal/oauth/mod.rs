//! OAuth/OIDC validation for remote MCP clients.

pub mod audit;
pub mod jwks;
pub mod validator;

pub use jwks::{Jwk, Jwks, JwksCache, JwksHttpClient, fetch_jwks};
pub use validator::{
    OAuthIssuerConfig, OAuthTokenClaims, PreparedOAuthVerifier, TokenAudience, ValidatedOAuthToken,
    validate_bearer_jwt,
};
