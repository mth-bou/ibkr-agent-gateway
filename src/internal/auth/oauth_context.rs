//! Remote OAuth/OIDC auth context mapping.

use super::{AuthContext, AuthContextSource, ScopeSet};
use crate::internal::domain::{
    AccountIdHash, ErrorCode, GatewayError, LocalUserId, RequestId, SessionId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use time::OffsetDateTime;

/// Input produced after token validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthContextInput {
    /// Subject.
    pub subject: String,
    /// Issuer.
    pub issuer: String,
    /// Matched audience.
    pub audience: String,
    /// Granted scopes.
    pub scopes: BTreeSet<String>,
    /// Token expiry.
    pub expires_at: OffsetDateTime,
    /// Token id hash.
    pub token_id_hash: Option<AccountIdHash>,
    /// Request id.
    pub request_id: RequestId,
    /// Session id.
    pub session_id: SessionId,
}

/// Remote OAuth auth context.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoteAuthContext {
    /// Gateway user id derived from issuer and subject.
    pub user_id: LocalUserId,
    /// Token subject.
    pub subject: String,
    /// Token issuer.
    pub issuer: String,
    /// Matched audience.
    pub audience: String,
    /// Granted gateway scopes.
    pub scopes: ScopeSet,
    /// Token expiry.
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    /// HMAC hash of token id when present.
    pub token_id_hash: Option<AccountIdHash>,
    /// Request correlation id.
    pub request_id: RequestId,
    /// Session correlation id.
    pub session_id: SessionId,
}

/// Builds a remote auth context after validating scopes against local policy.
pub fn remote_auth_context_from_input(
    input: OAuthContextInput,
) -> Result<RemoteAuthContext, GatewayError> {
    if input.subject.trim().is_empty() {
        return Err(GatewayError::new(
            ErrorCode::AuthTokenInvalid,
            "OAuth subject is empty",
            false,
            Some("Use a token with a stable subject".to_string()),
        ));
    }

    let user_id =
        LocalUserId::new(format!("{}:{}", input.issuer, input.subject)).ok_or_else(|| {
            GatewayError::new(
                ErrorCode::AuthTokenInvalid,
                "Remote user id could not be derived",
                false,
                Some("Use a token with issuer and subject".to_string()),
            )
        })?;
    let scopes = ScopeSet::local_with_live(input.scopes)?;

    Ok(RemoteAuthContext {
        user_id,
        subject: input.subject,
        issuer: input.issuer,
        audience: input.audience,
        scopes,
        expires_at: input.expires_at,
        token_id_hash: input.token_id_hash,
        request_id: input.request_id,
        session_id: input.session_id,
    })
}

impl From<RemoteAuthContext> for AuthContext {
    fn from(context: RemoteAuthContext) -> Self {
        Self {
            source: AuthContextSource::RemoteOauth,
            user_id: context.user_id,
            scopes: context.scopes,
            request_id: context.request_id,
            session_id: context.session_id,
        }
    }
}
