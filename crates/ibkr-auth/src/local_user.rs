//! Local user identity and authorization context.

use crate::scopes::ScopeSet;
use ibkr_domain::{LocalUserId, RequestId, SessionId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Auth context source supported by the read-only MVP.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthContextSource {
    /// Local configuration, not OAuth/OIDC.
    LocalConfig,
    /// Remote OAuth/OIDC bearer token.
    RemoteOauth,
}

/// Local user configured for the gateway.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct LocalUser {
    /// Stable local user id.
    pub user_id: LocalUserId,
    /// Optional display label.
    pub display_name: Option<String>,
    /// Allowed local read scopes.
    pub allowed_scopes: ScopeSet,
}

/// Authorization context attached to one operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AuthContext {
    /// Auth source.
    pub source: AuthContextSource,
    /// Local user id.
    pub user_id: LocalUserId,
    /// Granted scopes.
    pub scopes: ScopeSet,
    /// Request correlation id.
    pub request_id: RequestId,
    /// Session correlation id.
    pub session_id: SessionId,
}

impl AuthContext {
    /// Creates a local auth context.
    #[must_use]
    pub fn local(user: &LocalUser, request_id: RequestId, session_id: SessionId) -> Self {
        Self {
            source: AuthContextSource::LocalConfig,
            user_id: user.user_id.clone(),
            scopes: user.allowed_scopes.clone(),
            request_id,
            session_id,
        }
    }
}
