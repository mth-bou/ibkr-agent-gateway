//! Compatibility crate for MCP helpers while root migration is in progress.

#[path = "../../../src/internal/mcp/audit.rs"]
pub mod audit;
#[path = "../../../src/internal/mcp/http_auth.rs"]
pub mod http_auth;
#[path = "../../../src/internal/mcp/http_server.rs"]
pub mod http_server;
#[path = "../../../src/internal/mcp/keepalive.rs"]
pub mod keepalive;
#[path = "../../../src/internal/mcp/oauth_metadata.rs"]
pub mod oauth_metadata;
#[path = "../../../src/internal/mcp/registry.rs"]
pub mod registry;
#[path = "../../../src/internal/mcp/schemas.rs"]
pub mod schemas;
#[path = "../../../src/internal/mcp/scope_guard.rs"]
pub mod scope_guard;
#[path = "../../../src/internal/mcp/server.rs"]
pub mod server;
#[path = "../../../src/internal/mcp/session.rs"]
pub mod session;
#[path = "../../../src/internal/mcp/sidecar_relay.rs"]
pub mod sidecar_relay;
#[path = "../../../src/internal/mcp/tools/mod.rs"]
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
