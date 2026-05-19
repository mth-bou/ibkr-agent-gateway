//! Approval MCP tool schemas.

use crate::internal::auth::APPROVALS_CREATE;
use crate::internal::mcp::schemas::{ToolSchema, object_schema, safe_output_schema};

/// MCP approval creation tool.
pub const APPROVALS_CREATE_TOOL: &str = "ibkr_approvals_create";

/// Schema for creating a gateway approval record from an existing preview.
#[must_use]
pub fn approvals_create_schema() -> ToolSchema {
    ToolSchema {
        name: APPROVALS_CREATE_TOOL.to_string(),
        scope: APPROVALS_CREATE.to_string(),
        input_schema: object_schema(&["account_id", "preview_id", "ttl_seconds"]),
        output_schema: safe_output_schema(),
    }
}
