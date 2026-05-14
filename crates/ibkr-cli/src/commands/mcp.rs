//! MCP serve command.

use crate::output::print_output;
use ibkr_domain::{ErrorCode, GatewayError};
use serde::Serialize;

/// MCP serve output.
#[derive(Debug, Serialize)]
pub struct McpServeOutput {
    /// Transport name.
    pub transport: String,
    /// Status message.
    pub status: String,
}

/// Runs `ibkr-agent mcp serve --transport stdio`.
pub fn serve(transport: &str, json: bool) -> Result<(), GatewayError> {
    if transport != "stdio" {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            "Only stdio MCP transport is supported in the local MVP",
            false,
            Some("Use --transport stdio".to_string()),
        ));
    }
    let output = McpServeOutput {
        transport: transport.to_string(),
        status: ibkr_mcp::serve_stdio_description(),
    };
    print_output(json, &output.status, &output)
}
