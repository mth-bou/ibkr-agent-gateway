//! Provider compatibility harnesses and snapshots.

pub mod anthropic;
pub mod generic_mcp;
pub mod openai;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Provider under compatibility test.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderName {
    /// Generic MCP client or inspector.
    GenericMcp,
    /// OpenAI remote MCP.
    OpenAi,
    /// Anthropic MCP connector.
    Anthropic,
    /// Cursor local MCP client.
    Cursor,
    /// Continue local MCP client.
    Continue,
}

/// MCP client kind.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    /// MCP inspector.
    Inspector,
    /// Remote MCP connector.
    RemoteConnector,
    /// Local IDE client.
    LocalIde,
}

/// MCP transport kind.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompatTransport {
    /// Local stdio MCP.
    Stdio,
    /// Remote Streamable HTTP MCP.
    Http,
}

/// Authentication mode used by the provider/client.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CompatAuthMode {
    /// Local config scopes.
    LocalConfig,
    /// OAuth/OIDC bearer token.
    OAuthBearer,
}

/// Provider compatibility target.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProviderTarget {
    /// Provider name.
    pub provider: ProviderName,
    /// Client kind.
    pub client_kind: ClientKind,
    /// MCP transport.
    pub mcp_transport: CompatTransport,
    /// Auth mode.
    pub auth_mode: CompatAuthMode,
    /// Enabled feature labels.
    pub enabled_features: Vec<String>,
}

/// One compatibility scenario.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompatibilityScenario {
    /// Scenario id.
    pub scenario_id: String,
    /// Provider target.
    pub provider_target: ProviderTarget,
    /// Tool name.
    pub tool_name: String,
    /// Input fixture.
    pub input_fixture: serde_json::Value,
    /// Expected response shape label.
    pub expected_shape: String,
    /// Expected auth behavior label.
    pub expected_auth_behavior: String,
}

/// Redaction report for provider-facing payloads.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RedactionReport {
    /// Checked field paths.
    pub checked_fields: Vec<String>,
    /// Leaks found.
    pub leaks_found: Vec<String>,
    /// Redaction labels.
    pub redactions: Vec<String>,
    /// Secret scan result.
    pub secret_scan_result: String,
}

/// Compatibility snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CompatibilitySnapshot {
    /// Snapshot id.
    pub snapshot_id: String,
    /// Scenario id.
    pub scenario_id: String,
    /// Tool schema hash.
    pub tool_schema_hash: String,
    /// Response hash.
    pub response_hash: String,
    /// Redaction report.
    pub redaction_report: RedactionReport,
    /// Creation timestamp label.
    pub created_at: String,
}
