#[path = "common/remote_oauth.rs"]
mod remote_oauth;

use ibkr_auth::{ACCOUNTS_READ, HEALTH_READ};
use ibkr_config::validate_remote_mcp_config;
use ibkr_domain::ErrorCode;
use ibkr_mcp::http_server::{HttpMcpRequest, handle_http_mcp_request};
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime};

#[test]
fn remote_mcp_missing_token_returns_401() -> Result<(), Box<dyn std::error::Error>> {
    let request = HttpMcpRequest {
        path: "/mcp".to_string(),
        headers: BTreeMap::new(),
        tool_name: Some("ibkr_accounts_list".to_string()),
        body: serde_json::json!({}),
    };
    let response = handle_http_mcp_request(
        &remote_oauth::remote_config()?,
        Some(&remote_oauth::jwks()),
        &request,
    );

    assert_eq!(response.status, 401);
    assert!(response.headers.contains_key("www-authenticate"));
    assert_eq!(
        response.body["oauth_protected_resource"]["resource"],
        remote_oauth::AUDIENCE
    );
    Ok(())
}

#[test]
fn remote_mcp_requires_token_hash_secret_when_enabled() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = remote_oauth::remote_config()?;
    config.token_id_hmac_secret = None;

    let error = validate_remote_mcp_config(&config, true);
    let Err(error) = error else {
        return Err("remote MCP config without token hash secret should be rejected".into());
    };
    assert_eq!(error.code, ErrorCode::ConfigInvalid);
    assert!(error.message.contains("remote_mcp.token_id_hmac_secret"));
    Ok(())
}

#[test]
fn remote_mcp_valid_token_with_missing_scope_returns_403() -> Result<(), Box<dyn std::error::Error>>
{
    let token = remote_oauth::token(
        remote_oauth::ISSUER,
        remote_oauth::AUDIENCE,
        HEALTH_READ,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )?;
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), format!("Bearer {token}"));
    let request = HttpMcpRequest {
        path: "/mcp".to_string(),
        headers,
        tool_name: Some("ibkr_accounts_list".to_string()),
        body: serde_json::json!({}),
    };

    let response = handle_http_mcp_request(
        &remote_oauth::remote_config()?,
        Some(&remote_oauth::jwks()),
        &request,
    );

    assert_eq!(response.status, 403);
    assert_eq!(response.body["error"], "AUTH_MISSING_SCOPE");
    Ok(())
}

#[test]
fn remote_mcp_valid_token_with_scope_is_authorized() -> Result<(), Box<dyn std::error::Error>> {
    let token = remote_oauth::token(
        remote_oauth::ISSUER,
        remote_oauth::AUDIENCE,
        ACCOUNTS_READ,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )?;
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), format!("Bearer {token}"));
    let request = HttpMcpRequest {
        path: "/mcp".to_string(),
        headers,
        tool_name: Some("ibkr_accounts_list".to_string()),
        body: serde_json::json!({}),
    };

    let response = handle_http_mcp_request(
        &remote_oauth::remote_config()?,
        Some(&remote_oauth::jwks()),
        &request,
    );

    assert_eq!(response.status, 200);
    assert_eq!(response.body["status"], "authorized");
    assert_eq!(response.body["scope"], ACCOUNTS_READ);
    Ok(())
}
