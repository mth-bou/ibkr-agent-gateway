//! Streamable HTTP MCP transport facade.

use crate::{registry::broker_tool_schemas, session::HttpMcpSessionIds};
use ibkr_config::RemoteMcpConfig;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

/// Authorization header name.
pub const AUTHORIZATION_HEADER: &str = "authorization";
/// Protected-resource metadata path.
pub const PROTECTED_RESOURCE_METADATA_PATH: &str = "/.well-known/oauth-protected-resource";
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
    if !tools.iter().any(|tool| &tool.name == tool_name) {
        return HttpMcpResponse::json(
            404,
            json!({
                "error": "unknown_tool",
                "message": "MCP tool is not registered"
            }),
        );
    }

    HttpMcpResponse::json(
        501,
        json!({
            "error": "remote_auth_not_configured",
            "message": "Remote MCP HTTP transport is installed; OAuth enforcement is required before tool execution"
        }),
    )
}

/// Builds the protected resource metadata response.
#[must_use]
pub fn protected_resource_metadata_response(config: &RemoteMcpConfig) -> HttpMcpResponse {
    HttpMcpResponse::json(200, protected_resource_metadata(config))
}

/// Builds OAuth protected resource metadata.
#[must_use]
pub fn protected_resource_metadata(config: &RemoteMcpConfig) -> serde_json::Value {
    json!({
        "resource": config.resource.as_ref().map(ToString::to_string),
        "authorization_servers": config
            .metadata_url
            .as_ref()
            .map(|url| vec![url.to_string()])
            .unwrap_or_default(),
        "jwks_uri": config.jwks_url.as_ref().map(ToString::to_string),
        "scopes_supported": config.allowed_scopes,
    })
}
