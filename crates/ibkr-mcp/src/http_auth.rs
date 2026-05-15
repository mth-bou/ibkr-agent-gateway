//! Remote MCP HTTP auth enforcement.

use crate::{
    http_server::HttpMcpResponse, oauth_metadata::protected_resource_metadata,
    session::HttpMcpSessionIds,
};
use ibkr_auth::{OAuthContextInput, RemoteAuthContext, remote_auth_context_from_input};
use ibkr_config::RemoteMcpConfig;
use ibkr_domain::{ErrorCode, GatewayError};
use ibkr_oauth::{Jwks, OAuthIssuerConfig, validate_bearer_jwt};
use serde_json::json;
use std::collections::BTreeMap;
use time::OffsetDateTime;

/// Authorizes a remote MCP request before tool execution.
pub fn authorize_remote_request(
    config: &RemoteMcpConfig,
    jwks: &Jwks,
    headers: &BTreeMap<String, String>,
    required_scope: &str,
) -> Result<RemoteAuthContext, HttpMcpResponse> {
    let token =
        bearer_token(headers).ok_or_else(|| auth_error_response(config, missing_token()))?;
    let oauth_config =
        oauth_issuer_config(config).map_err(|error| auth_error_response(config, error))?;
    let validated = validate_bearer_jwt(
        token,
        &oauth_config,
        jwks,
        Some(required_scope),
        OffsetDateTime::now_utc(),
    )
    .map_err(|error| auth_error_response(config, error))?;
    let session_ids = HttpMcpSessionIds::from_headers(headers);
    let expires_at = OffsetDateTime::from_unix_timestamp(validated.claims.exp)
        .map_err(|_| auth_error_response(config, invalid_time()))?;

    remote_auth_context_from_input(OAuthContextInput {
        subject: validated.claims.sub,
        issuer: validated.claims.iss,
        audience: validated.audience,
        scopes: validated.granted_scopes,
        expires_at,
        token_id_hash: validated.token_id_hash,
        request_id: session_ids.request_id,
        session_id: session_ids.session_id,
    })
    .map_err(|error| auth_error_response(config, error))
}

fn oauth_issuer_config(config: &RemoteMcpConfig) -> Result<OAuthIssuerConfig, GatewayError> {
    let issuer = config.issuer.as_ref().ok_or_else(|| {
        GatewayError::new(
            ErrorCode::ConfigInvalid,
            "Remote MCP issuer is not configured",
            false,
            Some("Configure remote_mcp.issuer".to_string()),
        )
    })?;
    let jwks_url = config.jwks_url.as_ref().ok_or_else(|| {
        GatewayError::new(
            ErrorCode::ConfigInvalid,
            "Remote MCP JWKS URL is not configured",
            false,
            Some("Configure remote_mcp.jwks_url".to_string()),
        )
    })?;
    let token_id_hmac_secret = config
        .token_id_hmac_secret
        .as_deref()
        .filter(|secret| !secret.trim().is_empty())
        .ok_or_else(|| {
            GatewayError::new(
                ErrorCode::ConfigInvalid,
                "Remote MCP token id HMAC secret is not configured",
                false,
                Some("Configure remote_mcp.token_id_hmac_secret from a secret store".to_string()),
            )
        })?;

    Ok(OAuthIssuerConfig {
        issuer: issuer.to_string(),
        jwks_url: jwks_url.to_string(),
        audiences: config.audiences.clone(),
        allowed_scopes: config.allowed_scopes.clone(),
        clock_skew_seconds: config.clock_skew_seconds,
        metadata_url: config.metadata_url.as_ref().map(ToString::to_string),
        token_id_hmac_secret: token_id_hmac_secret.as_bytes().to_vec(),
    })
}

fn bearer_token(headers: &BTreeMap<String, String>) -> Option<&str> {
    let value = headers
        .get("authorization")
        .or_else(|| headers.get("Authorization"))?;
    value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .filter(|token| !token.trim().is_empty())
}

fn auth_error_response(config: &RemoteMcpConfig, error: GatewayError) -> HttpMcpResponse {
    let status = match error.code {
        ErrorCode::AuthMissingScope => 403,
        ErrorCode::AuthTokenMissing
        | ErrorCode::AuthTokenInvalid
        | ErrorCode::AuthTokenExpired
        | ErrorCode::AuthInvalidIssuer
        | ErrorCode::AuthInvalidAudience => 401,
        _ => 401,
    };
    let mut response = HttpMcpResponse::json(
        status,
        json!({
            "error": error.code,
            "message": error.message,
            "oauth_protected_resource": protected_resource_metadata(config)
        }),
    );
    if status == 401 {
        response.headers.insert(
            "www-authenticate".to_string(),
            "Bearer resource_metadata=\"/.well-known/oauth-protected-resource\"".to_string(),
        );
    }
    response
}

fn missing_token() -> GatewayError {
    GatewayError::new(
        ErrorCode::AuthTokenMissing,
        "Bearer token is required",
        false,
        Some("Send Authorization: Bearer <token>".to_string()),
    )
}

fn invalid_time() -> GatewayError {
    GatewayError::new(
        ErrorCode::AuthTokenInvalid,
        "JWT expiry timestamp is invalid",
        false,
        Some("Use a token with a valid exp claim".to_string()),
    )
}
