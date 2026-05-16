//! OAuth protected-resource metadata for remote MCP.

use crate::internal::config::RemoteMcpConfig;
use serde_json::json;

/// Protected-resource metadata path.
pub const PROTECTED_RESOURCE_METADATA_PATH: &str = "/.well-known/oauth-protected-resource";

/// Builds OAuth protected-resource metadata.
#[must_use]
pub fn protected_resource_metadata(config: &RemoteMcpConfig) -> serde_json::Value {
    json!({
        "resource": config.resource.as_ref().map(ToString::to_string),
        "authorization_servers": config
            .metadata_url
            .as_ref()
            .map(|url| vec![url.to_string()])
            .unwrap_or_default(),
        "jwks_uri": config.jwks_url.as_ref().map(ToString::to_string),
        "scopes_supported": config.allowed_scopes,
    })
}
