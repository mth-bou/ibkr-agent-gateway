//! OpenAI remote MCP compatibility harness.

use crate::{
    ClientKind, CompatAuthMode, CompatTransport, CompatibilityScenario, ProviderName,
    ProviderTarget,
};

/// Builds the OpenAI remote MCP target without depending on an OpenAI SDK.
#[must_use]
pub fn target() -> ProviderTarget {
    ProviderTarget {
        provider: ProviderName::OpenAi,
        client_kind: ClientKind::RemoteConnector,
        mcp_transport: CompatTransport::Http,
        auth_mode: CompatAuthMode::OAuthBearer,
        enabled_features: vec![
            "remote_mcp".to_string(),
            "oauth_protected_resource".to_string(),
        ],
    }
}

/// Builds an OpenAI Responses MCP compatibility scenario.
#[must_use]
pub fn scenario() -> CompatibilityScenario {
    CompatibilityScenario {
        scenario_id: "openai-remote-mcp-accounts-list".to_string(),
        provider_target: target(),
        tool_name: "ibkr_accounts_list".to_string(),
        input_fixture: serde_json::json!({}),
        expected_shape: "authorized_mcp_tool_response".to_string(),
        expected_auth_behavior: "oauth_bearer_required".to_string(),
    }
}
