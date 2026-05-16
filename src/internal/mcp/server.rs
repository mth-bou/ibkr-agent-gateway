//! MCP server entrypoints.

use super::registry::broker_tool_schemas;
use ibkr_config::RemoteMcpConfig;

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
        broker_tool_schemas().len()
    )
}

/// Describes the remote MCP HTTP server.
pub fn serve_http_description(
    config: &RemoteMcpConfig,
) -> Result<String, ibkr_domain::GatewayError> {
    ibkr_config::validate_remote_mcp_config(config, config.enabled)?;
    Ok(format!(
        "remote mcp http ready on {} with {} tools",
        config.bind_address,
        broker_tool_schemas().len()
    ))
}
