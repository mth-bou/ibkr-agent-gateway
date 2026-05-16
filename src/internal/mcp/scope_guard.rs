//! MCP local scope enforcement.

use ibkr_auth::{ScopeSet, require_scope};
use ibkr_domain::GatewayError;

/// Denies before broker access when a required local scope is missing.
pub fn enforce_scope(scopes: &ScopeSet, required_scope: &str) -> Result<(), GatewayError> {
    require_scope(scopes, required_scope)
}
