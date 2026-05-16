//! Anthropic MCP connector compatibility harness.

use super::{
    ClientKind, CompatAuthMode, CompatTransport, CompatibilityScenario, ProviderName,
    ProviderTarget,
};

/// Builds the Anthropic MCP connector target without depending on an Anthropic SDK.
#[must_use]
pub fn target() -> ProviderTarget {
    ProviderTarget {
        provider: ProviderName::Anthropic,
        client_kind: ClientKind::RemoteConnector,
        mcp_transport: CompatTransport::Http,
        auth_mode: CompatAuthMode::OAuthBearer,
        enabled_features: vec!["remote_mcp".to_string(), "scoped_tools".to_string()],
    }
}

/// Builds an Anthropic MCP connector compatibility scenario.
#[must_use]
pub fn scenario() -> CompatibilityScenario {
    CompatibilityScenario {
        scenario_id: "anthropic-remote-mcp-scope-denial".to_string(),
        provider_target: target(),
        tool_name: "ibkr_accounts_list".to_string(),
        input_fixture: serde_json::json!({}),
        expected_shape: "structured_auth_denial".to_string(),
        expected_auth_behavior: "insufficient_scope_403".to_string(),
    }
}
