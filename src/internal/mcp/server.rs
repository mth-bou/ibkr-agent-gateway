//! MCP server entrypoints.

use super::registry::local_tool_schemas;
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
        "local mcp stdio ready with {} scope-filtered tools",
        local_tool_schemas().len()
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
        local_tool_schemas().len()
    ))
}
