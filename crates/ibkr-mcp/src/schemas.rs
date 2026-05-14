//! MCP tool schema descriptions.

use serde::{Deserialize, Serialize};
use serde_json::json;

/// Minimal schema metadata for a local MCP tool.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name.
    pub name: String,
    /// Required scope.
    pub scope: String,
    /// Input JSON schema.
    pub input_schema: serde_json::Value,
    /// Output JSON schema.
    pub output_schema: serde_json::Value,
}

/// Creates a generic object schema for read-only tools.
#[must_use]
pub fn object_schema(required: &[&str]) -> serde_json::Value {
    json!({
        "type": "object",
        "required": required,
        "additionalProperties": false
    })
}

/// Creates a generic safe output schema.
#[must_use]
pub fn safe_output_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "additionalProperties": true,
        "x-redaction": "tokens,cookies,credentials,headers,local_paths"
    })
}
