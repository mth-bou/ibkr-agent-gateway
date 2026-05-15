#[path = "common/remote_oauth.rs"]
mod remote_oauth;

use ibkr_auth::HEALTH_READ;
use ibkr_mcp::http_server::{HttpMcpRequest, handle_http_mcp_request};
use serde_json::json;
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime};

#[test]
fn provider_missing_bearer_token_denial_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let response = handle_http_mcp_request(
        &remote_oauth::remote_config()?,
        Some(&remote_oauth::jwks()),
        &provider_request(BTreeMap::new()),
    );

    assert_eq!(response.status, 401);
    assert!(response.headers.contains_key("www-authenticate"));
    assert_eq!(
        json!({
            "status": response.status,
            "error": response.body["error"],
            "message": response.body["message"],
            "resource": response.body["oauth_protected_resource"]["resource"],
        }),
        json!({
            "status": 401,
            "error": "AUTH_TOKEN_MISSING",
            "message": "Bearer token is required",
            "resource": remote_oauth::AUDIENCE,
        })
    );

    Ok(())
}

#[test]
fn provider_missing_scope_denial_snapshot() -> Result<(), Box<dyn std::error::Error>> {
    let token = remote_oauth::token(
        remote_oauth::ISSUER,
        remote_oauth::AUDIENCE,
        HEALTH_READ,
        OffsetDateTime::now_utc() + Duration::minutes(5),
    )?;
    let mut headers = BTreeMap::new();
    headers.insert("authorization".to_string(), format!("Bearer {token}"));

    let response = handle_http_mcp_request(
        &remote_oauth::remote_config()?,
        Some(&remote_oauth::jwks()),
        &provider_request(headers),
    );

    assert_eq!(
        json!({
            "status": response.status,
            "error": response.body["error"],
            "message": response.body["message"],
        }),
        json!({
            "status": 403,
            "error": "AUTH_MISSING_SCOPE",
            "message": "Missing required scope: ibkr:accounts:read",
        })
    );

    Ok(())
}

fn provider_request(headers: BTreeMap<String, String>) -> HttpMcpRequest {
    HttpMcpRequest {
        path: "/mcp".to_string(),
        headers,
        tool_name: Some("ibkr_accounts_list".to_string()),
        body: json!({}),
    }
}
