//! Local read-only scopes for the MVP.

use crate::internal::domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Health and broker session read scope.
pub const HEALTH_READ: &str = "ibkr:health:read";
/// Account listing read scope.
pub const ACCOUNTS_READ: &str = "ibkr:accounts:read";
/// Portfolio summary read scope.
pub const PORTFOLIO_READ: &str = "ibkr:portfolio:read";
/// Positions read scope.
pub const POSITIONS_READ: &str = "ibkr:positions:read";
/// Market data read scope.
pub const MARKETDATA_READ: &str = "ibkr:marketdata:read";
/// Read-only orders scope.
pub const ORDERS_READ: &str = "ibkr:orders:read";
/// Audit read scope.
pub const AUDIT_READ: &str = "ibkr:audit:read";
/// Order preview scope.
pub const ORDERS_PREVIEW: &str = "ibkr:orders:preview";
/// Risk policy read scope.
pub const RISK_READ: &str = "ibkr:risk:read";
/// Paper order submit scope.
pub const ORDERS_PAPER_SUBMIT: &str = "ibkr:orders:paper:submit";
/// Paper order cancel scope.
pub const ORDERS_PAPER_CANCEL: &str = "ibkr:orders:paper:cancel";
/// Live order submit scope.
pub const ORDERS_LIVE_SUBMIT: &str = "ibkr:orders:live:submit";
/// Live order cancel scope.
pub const ORDERS_LIVE_CANCEL: &str = "ibkr:orders:live:cancel";

/// All read scopes allowed in the MVP.
pub const READ_SCOPES: &[&str] = &[
    HEALTH_READ,
    ACCOUNTS_READ,
    PORTFOLIO_READ,
    POSITIONS_READ,
    MARKETDATA_READ,
    ORDERS_READ,
    AUDIT_READ,
];

/// Write-adjacent preview scopes allowed only after spec 002.
pub const PREVIEW_SCOPES: &[&str] = &[ORDERS_PREVIEW, RISK_READ];

/// Paper trading scopes allowed only after spec 003.
pub const PAPER_SCOPES: &[&str] = &[ORDERS_PAPER_SUBMIT, ORDERS_PAPER_CANCEL];

/// Live trading scopes allowed only after spec 007.
pub const LIVE_SCOPES: &[&str] = &[ORDERS_LIVE_SUBMIT, ORDERS_LIVE_CANCEL];

/// All local scopes known through spec 002.
pub const LOCAL_SCOPES: &[&str] = &[
    HEALTH_READ,
    ACCOUNTS_READ,
    PORTFOLIO_READ,
    POSITIONS_READ,
    MARKETDATA_READ,
    ORDERS_READ,
    AUDIT_READ,
    ORDERS_PREVIEW,
    RISK_READ,
    ORDERS_PAPER_SUBMIT,
    ORDERS_PAPER_CANCEL,
    ORDERS_LIVE_SUBMIT,
    ORDERS_LIVE_CANCEL,
];

/// Set of local scopes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScopeSet {
    scopes: BTreeSet<String>,
}

impl ScopeSet {
    /// Creates a read-only scope set after validating all scopes.
    pub fn read_only(
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, GatewayError> {
        let scopes = scopes
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<String>>();

        if let Some(scope) = scopes.iter().find(|scope| !is_read_scope(scope)) {
            return Err(GatewayError::new(
                ErrorCode::AuthScopeNotAllowedInMvp,
                format!("Scope is not allowed in read-only MVP: {scope}"),
                false,
                Some("Remove write, remote, sidecar, or live scopes".to_string()),
            ));
        }

        Ok(Self { scopes })
    }

    /// Creates a local scope set that may include preview scopes.
    pub fn local_with_preview(
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, GatewayError> {
        Self::local(scopes)
    }

    /// Creates a local scope set that may include paper scopes.
    pub fn local_with_paper(
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, GatewayError> {
        Self::local(scopes)
    }

    /// Creates a local scope set that may include live trading scopes.
    pub fn local_with_live(
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, GatewayError> {
        Self::local(scopes)
    }

    fn local(scopes: impl IntoIterator<Item = impl Into<String>>) -> Result<Self, GatewayError> {
        let scopes = scopes
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<String>>();

        if let Some(scope) = scopes.iter().find(|scope| !is_local_scope(scope)) {
            return Err(GatewayError::new(
                ErrorCode::AuthScopeNotAllowedInMvp,
                format!("Scope is not allowed locally: {scope}"),
                false,
                Some("Remove remote, sidecar, submit, cancel, or live scopes".to_string()),
            ));
        }

        Ok(Self { scopes })
    }

    /// Returns whether the set includes the required scope.
    #[must_use]
    pub fn contains(&self, required_scope: &str) -> bool {
        self.scopes.contains(required_scope)
    }

    /// Returns scopes as a sorted set.
    #[must_use]
    pub const fn as_set(&self) -> &BTreeSet<String> {
        &self.scopes
    }
}

/// Returns true when the scope is valid for the read-only MVP.
#[must_use]
pub fn is_read_scope(scope: &str) -> bool {
    READ_SCOPES.contains(&scope)
}

/// Returns true when the scope is valid for the local gateway through spec 002.
#[must_use]
pub fn is_local_scope(scope: &str) -> bool {
    LOCAL_SCOPES.contains(&scope)
}

/// Ensures a scope is present before a broker call.
pub fn require_scope(scopes: &ScopeSet, required_scope: &str) -> Result<(), GatewayError> {
    if scopes.contains(required_scope) {
        Ok(())
    } else {
        Err(GatewayError::new(
            ErrorCode::AuthMissingScope,
            format!("Missing required scope: {required_scope}"),
            false,
            Some("Enable the required local read scope".to_string()),
        ))
    }
}
