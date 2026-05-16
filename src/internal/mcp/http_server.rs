//! Streamable HTTP MCP transport facade.

use super::{
    http_auth::authorize_remote_request,
    oauth_metadata::{PROTECTED_RESOURCE_METADATA_PATH, protected_resource_metadata},
    registry::broker_tool_schemas,
    session::HttpMcpSessionIds,
};
use ibkr_config::RemoteMcpConfig;
use ibkr_oauth::Jwks;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

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
    let tools = broker_tool_schemas();
    let Some(tool_name) = &request.tool_name else {
        return HttpMcpResponse::json(
            400,
            json!({
                "error": "missing_tool",
                "message": "MCP tool name is required"
            }),
        );
    };
    let Some(tool) = tools.iter().find(|tool| &tool.name == tool_name) else {
        return HttpMcpResponse::json(
            404,
            json!({
                "error": "unknown_tool",
                "message": "MCP tool is not registered"
            }),
        );
    };
    let Some(jwks) = jwks else {
        return HttpMcpResponse::json(
            401,
            json!({
                "error": "jwks_unavailable",
                "message": "Remote MCP cannot validate tokens without JWKS"
            }),
        );
    };
    if let Err(response) = authorize_remote_request(config, jwks, &request.headers, &tool.scope) {
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

/// Builds the protected resource metadata response.
#[must_use]
pub fn protected_resource_metadata_response(config: &RemoteMcpConfig) -> HttpMcpResponse {
    HttpMcpResponse::json(200, protected_resource_metadata(config))
}
