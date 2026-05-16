//! Audit MCP tool names and helpers.

use super::super::schemas::{ToolSchema, object_schema, safe_output_schema};
use ibkr_auth::AUDIT_READ;

/// Audit tail tool.
pub const AUDIT_TAIL_TOOL: &str = "ibkr_audit_tail";

/// Schema for the audit tail tool.
#[must_use]
pub fn audit_tail_schema() -> ToolSchema {
    ToolSchema {
        name: AUDIT_TAIL_TOOL.to_string(),
        scope: AUDIT_READ.to_string(),
        input_schema: object_schema(&["limit"]),
        output_schema: safe_output_schema(),
    }
}
