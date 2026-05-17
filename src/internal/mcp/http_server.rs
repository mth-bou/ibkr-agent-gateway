//! Streamable HTTP MCP transport facade.

use super::{
    http_auth::{RemoteMcpAuthVerifier, authorize_remote_request_with_verifier},
    oauth_metadata::{PROTECTED_RESOURCE_METADATA_PATH, protected_resource_metadata},
    registry::find_broker_tool_schema,
    session::HttpMcpSessionIds,
};
use crate::internal::config::RemoteMcpConfig;
use crate::internal::oauth::Jwks;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

/// Prepared remote MCP runtime state for repeated HTTP requests.
#[derive(Clone, Debug)]
pub struct HttpMcpRuntime {
    auth_verifier: RemoteMcpAuthVerifier,
}

impl HttpMcpRuntime {
    /// Builds a prepared runtime from remote MCP config and JWKS.
    pub fn new(
        config: &RemoteMcpConfig,
        jwks: &Jwks,
    ) -> Result<Self, crate::internal::domain::GatewayError> {
        Ok(Self {
            auth_verifier: RemoteMcpAuthVerifier::new(config, jwks)?,
        })
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
    if find_broker_tool_schema(tool_name).is_none() {
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
    let Some(tool) = find_broker_tool_schema(tool_name) else {
        return HttpMcpResponse::json(
            404,
            json!({
                "error": "unknown_tool",
                "message": "MCP tool is not registered"
            }),
        );
    };
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

/// Builds the protected resource metadata response.
#[must_use]
pub fn protected_resource_metadata_response(config: &RemoteMcpConfig) -> HttpMcpResponse {
    HttpMcpResponse::json(200, protected_resource_metadata(config))
}
