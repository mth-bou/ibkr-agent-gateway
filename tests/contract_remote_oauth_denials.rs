#[path = "common/remote_oauth.rs"]
mod remote_oauth;

use ibkr_agent_gateway::testing::auth::{ACCOUNTS_READ, HEALTH_READ, ORDERS_LIVE_SUBMIT};
use ibkr_agent_gateway::testing::config::validate_remote_mcp_config;
use ibkr_agent_gateway::testing::domain::ErrorCode;
use ibkr_agent_gateway::testing::mcp::http_server::{
    HttpMcpRequest, HttpMcpRuntime, handle_http_mcp_request, handle_http_mcp_request_with_runtime,
};
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

#[test]
fn remote_mcp_can_authorize_live_tool_when_live_scope_is_allowed()
-> Result<(), Box<dyn std::error::Error>> {
    let mut config = remote_oauth::remote_config()?;
    config.allowed_scopes.push(ORDERS_LIVE_SUBMIT.to_string());
    let token = remote_oauth::token(
        remote_oauth::ISSUER,
        remote_oauth::AUDIENCE,
        ORDERS_LIVE_SUBMIT,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )?;
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), format!("Bearer {token}"));
    let request = HttpMcpRequest {
        path: "/mcp".to_string(),
        headers,
        tool_name: Some("ibkr_live_order_submit".to_string()),
        body: serde_json::json!({}),
    };

    let response = handle_http_mcp_request(&config, Some(&remote_oauth::jwks()), &request);

    assert_eq!(response.status, 200);
    assert_eq!(response.body["status"], "authorized");
    assert_eq!(response.body["scope"], ORDERS_LIVE_SUBMIT);
    Ok(())
}

#[test]
fn remote_mcp_rejects_malformed_bearer_before_crypto() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), "Bearer not-a-jwt".to_string());
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

    assert_eq!(response.status, 401);
    assert_eq!(response.body["message"], "Authentication failed");
    Ok(())
}

#[test]
fn remote_mcp_authorization_header_is_case_insensitive() -> Result<(), Box<dyn std::error::Error>> {
    let token = remote_oauth::token(
        remote_oauth::ISSUER,
        remote_oauth::AUDIENCE,
        ACCOUNTS_READ,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )?;
    let mut headers = BTreeMap::new();
    headers.insert("AUTHORIZATION".to_string(), format!("Bearer {token}"));
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
    Ok(())
}

#[test]
fn remote_mcp_runtime_rate_limits_repeated_authorization_attempts()
-> Result<(), Box<dyn std::error::Error>> {
    let config = remote_oauth::remote_config()?;
    let jwks = remote_oauth::jwks();
    let runtime = HttpMcpRuntime::new(&config, &jwks)?;
    let token = remote_oauth::token(
        remote_oauth::ISSUER,
        remote_oauth::AUDIENCE,
        ACCOUNTS_READ,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )?;
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), format!("Bearer {token}"));
    headers.insert("x-forwarded-for".to_string(), "203.0.113.10".to_string());
    let request = HttpMcpRequest {
        path: "/mcp".to_string(),
        headers,
        tool_name: Some("ibkr_accounts_list".to_string()),
        body: serde_json::json!({}),
    };

    for _ in 0..120 {
        let response = handle_http_mcp_request_with_runtime(&config, &runtime, &request);
        assert_eq!(response.status, 200);
    }

    let response = handle_http_mcp_request_with_runtime(&config, &runtime, &request);
    assert_eq!(response.status, 429);
    assert_eq!(response.body["error"], "rate_limited");
    Ok(())
}
