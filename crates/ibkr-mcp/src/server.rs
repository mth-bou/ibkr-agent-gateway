//! Local MCP stdio server entrypoint.

use crate::registry::broker_tool_schemas;

/// Supported MCP transports for the MVP.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpTransport {
    /// Local stdio transport.
    Stdio,
}

/// Starts the local MCP stdio server.
#[must_use]
pub fn serve_stdio_description() -> String {
    format!(
        "local mcp stdio ready with {} read-only tools",
        broker_tool_schemas().len()
    )
}
