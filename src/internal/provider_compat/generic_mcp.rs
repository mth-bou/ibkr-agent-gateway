//! Generic MCP inspector compatibility harness.

use super::{
    ClientKind, CompatAuthMode, CompatTransport, CompatibilityScenario, CompatibilitySnapshot,
    ProviderName, ProviderTarget, RedactionReport,
};
use crate::internal::encoding::sha256_hex;
use crate::internal::mcp::broker_tool_schemas_ref;

/// Builds the generic MCP inspector target.
#[must_use]
pub fn target() -> ProviderTarget {
    ProviderTarget {
        provider: ProviderName::GenericMcp,
        client_kind: ClientKind::Inspector,
        mcp_transport: CompatTransport::Stdio,
        auth_mode: CompatAuthMode::LocalConfig,
        enabled_features: vec!["tool_list".to_string(), "schema_validation".to_string()],
    }
}

/// Builds generic MCP inspector scenarios.
#[must_use]
pub fn scenarios() -> Vec<CompatibilityScenario> {
    broker_tool_schemas_ref()
        .iter()
        .map(|tool| CompatibilityScenario {
            scenario_id: format!("generic-mcp-{}", tool.name),
            provider_target: target(),
            tool_name: tool.name.clone(),
            input_fixture: serde_json::json!({}),
            expected_shape: "mcp_tool_schema_object".to_string(),
            expected_auth_behavior: "local_scope_required".to_string(),
        })
        .collect()
}

/// Builds deterministic schema snapshots for all discovered broker tools.
#[must_use]
pub fn schema_snapshots() -> Vec<CompatibilitySnapshot> {
    broker_tool_schemas_ref()
        .iter()
        .map(|tool| {
            let schema = serde_json::json!({
                "name": tool.name,
                "scope": tool.scope,
                "input_schema": tool.input_schema,
                "output_schema": tool.output_schema
            });
            let schema_bytes = serde_json::to_vec(&schema).unwrap_or_default();
            CompatibilitySnapshot {
                snapshot_id: format!("schema-{}", schema["name"].as_str().unwrap_or("unknown")),
                scenario_id: format!(
                    "generic-mcp-{}",
                    schema["name"].as_str().unwrap_or("unknown")
                ),
                tool_schema_hash: sha256_hex(&schema_bytes),
                response_hash: sha256_hex(b"schema-only"),
                redaction_report: RedactionReport {
                    checked_fields: vec!["output_schema.x-redaction".to_string()],
                    leaks_found: Vec::new(),
                    redactions: vec![
                        "tokens".to_string(),
                        "cookies".to_string(),
                        "credentials".to_string(),
                    ],
                    secret_scan_result: "pass".to_string(),
                },
                created_at: "fixture".to_string(),
            }
        })
        .collect()
}
