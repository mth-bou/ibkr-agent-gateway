//! MCP serve command.

use crate::cli::output::print_output;
use ibkr_config::RemoteMcpConfig;
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

/// Runs `ibkr-agent mcp serve`.
pub fn serve(
    transport: &str,
    enable_remote_mcp: bool,
    bind: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let status = match transport {
        "stdio" => ibkr_mcp::serve_stdio_description(),
        "http" => {
            if !enable_remote_mcp {
                return Err(GatewayError::new(
                    ErrorCode::ConfigRemoteMcpForbidden,
                    "HTTP MCP requires explicit remote enablement",
                    false,
                    Some("Pass --enable-remote-mcp with complete OAuth configuration".to_string()),
                ));
            }
            let config = RemoteMcpConfig {
                enabled: true,
                bind_address: bind.to_string(),
                ..RemoteMcpConfig::default()
            };
            ibkr_mcp::serve_http_description(&config)?
        }
        _ => {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                "Unsupported MCP transport",
                false,
                Some("Use --transport stdio or --transport http".to_string()),
            ));
        }
    };
    let output = McpServeOutput {
        transport: transport.to_string(),
        status,
    };
    print_output(json, &output.status, &output)
}
