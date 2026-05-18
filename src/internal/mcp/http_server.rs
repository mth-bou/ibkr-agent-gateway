//! Streamable HTTP MCP transport facade.

use super::{
    http_auth::{RemoteMcpAuthVerifier, authorize_remote_request_with_verifier},
    oauth_metadata::{PROTECTED_RESOURCE_METADATA_PATH, protected_resource_metadata},
    registry::find_broker_tool_schema_with_live,
    session::HttpMcpSessionIds,
};
use crate::internal::oauth::Jwks;
use crate::internal::{
    auth::{ORDERS_LIVE_CANCEL, ORDERS_LIVE_SUBMIT},
    config::RemoteMcpConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;

/// Prepared remote MCP runtime state for repeated HTTP requests.
#[derive(Clone, Debug)]
pub struct HttpMcpRuntime {
    auth_verifier: RemoteMcpAuthVerifier,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl HttpMcpRuntime {
    /// Builds a prepared runtime from remote MCP config and JWKS.
    pub fn new(
        config: &RemoteMcpConfig,
        jwks: &Jwks,
    ) -> Result<Self, crate::internal::domain::GatewayError> {
        Ok(Self {
            auth_verifier: RemoteMcpAuthVerifier::new(config, jwks)?,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::from_config(config))),
        })
    }
}

/// Shared HTTP MCP rate limiter for concrete transport adapters.
#[derive(Clone, Debug)]
pub struct HttpMcpRateLimiter {
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl HttpMcpRateLimiter {
    /// Builds a rate limiter from remote MCP configuration.
    #[must_use]
    pub fn from_config(config: &RemoteMcpConfig) -> Self {
        Self {
            rate_limiter: Arc::new(Mutex::new(RateLimiter::from_config(config))),
        }
    }

    /// Returns whether the request headers are still within the remote MCP rate limit.
    #[must_use]
    pub fn allows(&self, headers: &BTreeMap<String, String>) -> bool {
        rate_limit_allows(&self.rate_limiter, headers)
    }
}

impl Default for HttpMcpRateLimiter {
    fn default() -> Self {
        Self::from_config(&RemoteMcpConfig::default())
    }
}

#[derive(Clone, Copy, Debug)]
struct RateLimitConfig {
    window_seconds: i64,
    max_requests: u32,
}

impl RateLimitConfig {
    fn from_config(config: &RemoteMcpConfig) -> Self {
        Self {
            window_seconds: i64::try_from(config.rate_limit_window_seconds).unwrap_or(i64::MAX),
            max_requests: config.rate_limit_max_requests,
        }
    }
}

#[derive(Clone, Debug)]
struct RateLimitBucket {
    window_started_at: i64,
    request_count: u32,
}

#[derive(Clone, Debug)]
struct RateLimiter {
    buckets: BTreeMap<String, RateLimitBucket>,
    config: RateLimitConfig,
}

impl RateLimiter {
    fn from_config(config: &RemoteMcpConfig) -> Self {
        Self {
            buckets: BTreeMap::new(),
            config: RateLimitConfig::from_config(config),
        }
    }

    fn allow(&mut self, key: String, now_unix: i64) -> bool {
        self.prune(now_unix);
        let bucket = self.buckets.entry(key).or_insert(RateLimitBucket {
            window_started_at: now_unix,
            request_count: 0,
        });

        if now_unix - bucket.window_started_at >= self.config.window_seconds {
            bucket.window_started_at = now_unix;
            bucket.request_count = 0;
        }
        if bucket.request_count >= self.config.max_requests {
            return false;
        }
        bucket.request_count += 1;
        true
    }

    fn prune(&mut self, now_unix: i64) {
        self.buckets
            .retain(|_, bucket| now_unix - bucket.window_started_at < self.config.window_seconds);
    }
}

/// Authorization header name.
pub const AUTHORIZATION_HEADER: &str = "authorization";
/// MCP HTTP endpoint path.
pub const MCP_HTTP_PATH: &str = "/mcp";

/// Minimal HTTP request model used by tests and adapters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HttpMcpRequest {
    /// Request path.
    pub path: String,
    /// Lowercase or original-case headers.
    pub headers: BTreeMap<String, String>,
    /// Optional MCP tool name.
    pub tool_name: Option<String>,
    /// JSON request body.
    pub body: serde_json::Value,
}

/// Minimal HTTP response model returned by the transport facade.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HttpMcpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: BTreeMap<String, String>,
    /// JSON response body.
    pub body: serde_json::Value,
}

impl HttpMcpResponse {
    /// Builds a JSON response.
    #[must_use]
    pub fn json(status: u16, body: serde_json::Value) -> Self {
        let mut headers = BTreeMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        Self {
            status,
            headers,
            body,
        }
    }
}

/// Handles a remote MCP HTTP request without coupling to a specific web framework.
#[must_use]
pub fn handle_http_mcp_request(
    config: &RemoteMcpConfig,
    jwks: Option<&Jwks>,
    request: &HttpMcpRequest,
) -> HttpMcpResponse {
    if request.path == PROTECTED_RESOURCE_METADATA_PATH {
        return protected_resource_metadata_response(config);
    }

    if !config.enabled {
        return HttpMcpResponse::json(
            404,
            json!({
                "error": "remote_mcp_disabled",
                "message": "Remote MCP is disabled"
            }),
        );
    }

    let _ids = HttpMcpSessionIds::from_headers(&request.headers);
    let Some(tool_name) = &request.tool_name else {
        return HttpMcpResponse::json(
            400,
            json!({
                "error": "missing_tool",
                "message": "MCP tool name is required"
            }),
        );
    };
    if find_broker_tool_schema_with_live(tool_name, live_tools_enabled(config)).is_none() {
        return HttpMcpResponse::json(
            404,
            json!({
                "error": "unknown_tool",
                "message": "MCP tool is not registered"
            }),
        );
    }
    let Some(jwks) = jwks else {
        return HttpMcpResponse::json(
            401,
            json!({
                "error": "jwks_unavailable",
                "message": "Remote MCP cannot validate tokens without JWKS"
            }),
        );
    };
    let Ok(runtime) = HttpMcpRuntime::new(config, jwks) else {
        return HttpMcpResponse::json(
            401,
            json!({
                "error": "jwks_unavailable",
                "message": "Remote MCP cannot prepare token validation"
            }),
        );
    };

    handle_http_mcp_request_with_runtime(config, &runtime, request)
}

/// Handles a remote MCP HTTP request using prepared runtime state.
#[must_use]
pub fn handle_http_mcp_request_with_runtime(
    config: &RemoteMcpConfig,
    runtime: &HttpMcpRuntime,
    request: &HttpMcpRequest,
) -> HttpMcpResponse {
    if request.path == PROTECTED_RESOURCE_METADATA_PATH {
        return protected_resource_metadata_response(config);
    }

    if !config.enabled {
        return HttpMcpResponse::json(
            404,
            json!({
                "error": "remote_mcp_disabled",
                "message": "Remote MCP is disabled"
            }),
        );
    }

    let _ids = HttpMcpSessionIds::from_headers(&request.headers);
    let Some(tool_name) = &request.tool_name else {
        return HttpMcpResponse::json(
            400,
            json!({
                "error": "missing_tool",
                "message": "MCP tool name is required"
            }),
        );
    };
    let Some(tool) = find_broker_tool_schema_with_live(tool_name, live_tools_enabled(config))
    else {
        return HttpMcpResponse::json(
            404,
            json!({
                "error": "unknown_tool",
                "message": "MCP tool is not registered"
            }),
        );
    };
    if !has_structurally_valid_bearer_jwt(&request.headers) {
        return bearer_token_invalid_response();
    }
    if !runtime_rate_limit_allows(runtime, &request.headers) {
        return HttpMcpResponse::json(
            429,
            json!({
                "error": "rate_limited",
                "message": "Too many remote MCP authorization attempts"
            }),
        );
    }
    if let Err(response) = authorize_remote_request_with_verifier(
        config,
        &runtime.auth_verifier,
        &request.headers,
        &tool.scope,
    ) {
        return response;
    }

    HttpMcpResponse::json(
        200,
        json!({
            "status": "authorized",
            "tool_name": tool.name,
            "scope": tool.scope
        }),
    )
}

fn live_tools_enabled(config: &RemoteMcpConfig) -> bool {
    config
        .allowed_scopes
        .iter()
        .any(|scope| scope == ORDERS_LIVE_SUBMIT || scope == ORDERS_LIVE_CANCEL)
}

/// Builds the protected resource metadata response.
#[must_use]
pub fn protected_resource_metadata_response(config: &RemoteMcpConfig) -> HttpMcpResponse {
    HttpMcpResponse::json(200, protected_resource_metadata(config))
}

fn has_structurally_valid_bearer_jwt(headers: &BTreeMap<String, String>) -> bool {
    let Some(header) = header_value(headers, AUTHORIZATION_HEADER) else {
        return true;
    };
    let Some(token) = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
    else {
        return true;
    };
    token.split('.').count() == 3
}

fn bearer_token_invalid_response() -> HttpMcpResponse {
    let mut response = HttpMcpResponse::json(
        401,
        json!({
            "error": "AUTH_TOKEN_INVALID",
            "message": "Authentication failed"
        }),
    );
    response.headers.insert(
        "www-authenticate".to_string(),
        "Bearer resource_metadata=\"/.well-known/oauth-protected-resource\"".to_string(),
    );
    response
}

fn runtime_rate_limit_allows(runtime: &HttpMcpRuntime, headers: &BTreeMap<String, String>) -> bool {
    rate_limit_allows(&runtime.rate_limiter, headers)
}

fn rate_limit_allows(
    rate_limiter: &Arc<Mutex<RateLimiter>>,
    headers: &BTreeMap<String, String>,
) -> bool {
    let key = rate_limit_key(headers);
    let now_unix = OffsetDateTime::now_utc().unix_timestamp();
    let mut limiter = rate_limiter
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    limiter.allow(key, now_unix)
}

fn rate_limit_key(headers: &BTreeMap<String, String>) -> String {
    header_value(headers, "x-forwarded-for")
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| header_value(headers, "mcp-session-id"))
        .unwrap_or("anonymous")
        .to_string()
}

fn header_value<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
