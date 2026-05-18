//! Provider-neutral MCP tool registry and transports.

pub mod audit;
pub mod http_auth;
pub mod http_server;
pub mod keepalive;
pub mod live_orders;
pub mod oauth_metadata;
pub mod order_workflows;
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
    FORBIDDEN_TOOL_NAMES, broker_tool_schemas, broker_tool_schemas_ref,
    broker_tool_schemas_with_live, find_broker_tool_schema_with_live, find_local_tool_schema,
    is_forbidden_tool_name, local_tool_schemas, local_tool_schemas_for_scopes,
    refuse_forbidden_tool,
};
pub use schemas::ToolSchema;
pub use scope_guard::enforce_scope;
pub use server::{McpTransport, serve_http_description, serve_stdio_description};
pub use session::HttpMcpSessionIds;
