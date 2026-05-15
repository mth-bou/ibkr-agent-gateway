//! Provider-neutral MCP tool registry and transports.

pub mod audit;
pub mod http_auth;
pub mod http_server;
pub mod keepalive;
pub mod oauth_metadata;
pub mod registry;
pub mod schemas;
pub mod scope_guard;
pub mod server;
pub mod session;
pub mod sidecar_relay;
pub mod tools;

pub use audit::build_mcp_tool_event;
pub use keepalive::keepalive_once;
pub use registry::{
    FORBIDDEN_TOOL_NAMES, broker_tool_schemas, broker_tool_schemas_with_live,
    is_forbidden_tool_name, refuse_forbidden_tool,
};
pub use schemas::ToolSchema;
pub use scope_guard::enforce_scope;
pub use server::{McpTransport, serve_http_description, serve_stdio_description};
pub use session::HttpMcpSessionIds;
