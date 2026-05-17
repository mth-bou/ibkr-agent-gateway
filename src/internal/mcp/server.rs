//! MCP server entrypoints.

use super::registry::broker_tool_schema_count;
use crate::internal::config::RemoteMcpConfig;

/// Supported MCP transports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpTransport {
    /// Local stdio transport.
    Stdio,
    /// Remote Streamable HTTP transport.
    Http,
}

/// Starts the local MCP stdio server.
#[must_use]
pub fn serve_stdio_description() -> String {
    format!(
        "local mcp stdio ready with {} read-only tools",
        broker_tool_schema_count()
    )
}

/// Describes the remote MCP HTTP server.
pub fn serve_http_description(
    config: &RemoteMcpConfig,
) -> Result<String, crate::internal::domain::GatewayError> {
    crate::internal::config::validate_remote_mcp_config(config, config.enabled)?;
    Ok(format!(
        "remote mcp http ready on {} with {} tools",
        config.bind_address,
        broker_tool_schema_count()
    ))
}
