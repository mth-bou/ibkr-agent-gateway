//! MCP local scope enforcement.

use crate::internal::auth::{ScopeSet, require_scope};
use crate::internal::domain::GatewayError;

/// Denies before broker access when a required local scope is missing.
pub fn enforce_scope(scopes: &ScopeSet, required_scope: &str) -> Result<(), GatewayError> {
    require_scope(scopes, required_scope)
}
