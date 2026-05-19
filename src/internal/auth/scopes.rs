//! Local and remote gateway scopes.

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
/// Audit export scope.
pub const AUDIT_EXPORT: &str = "ibkr:audit:export";
/// Order preview scope.
pub const ORDERS_PREVIEW: &str = "ibkr:orders:preview";
/// Risk policy read scope.
pub const RISK_READ: &str = "ibkr:risk:read";
/// Paper order submit scope.
pub const ORDERS_PAPER_SUBMIT: &str = "ibkr:orders:paper:submit";
/// Paper order cancel scope.
pub const ORDERS_PAPER_CANCEL: &str = "ibkr:orders:paper:cancel";
/// Paper order modify scope.
pub const ORDERS_PAPER_MODIFY: &str = "ibkr:orders:paper:modify";
/// Live order submit scope.
pub const ORDERS_LIVE_SUBMIT: &str = "ibkr:orders:live:submit";
/// Live order cancel scope.
pub const ORDERS_LIVE_CANCEL: &str = "ibkr:orders:live:cancel";
/// Live order modify scope.
pub const ORDERS_LIVE_MODIFY: &str = "ibkr:orders:live:modify";
/// Options read scope.
pub const OPTIONS_READ: &str = "ibkr:options:read";
/// Market depth read scope.
pub const MARKETDATA_DEPTH_READ: &str = "ibkr:marketdata:depth:read";
/// Scanner read scope.
pub const SCANNER_READ: &str = "ibkr:scanner:read";
/// News read scope.
pub const NEWS_READ: &str = "ibkr:news:read";
/// Fundamentals read scope.
pub const FUNDAMENTALS_READ: &str = "ibkr:fundamentals:read";
/// Market calendar read scope.
pub const CALENDAR_READ: &str = "ibkr:calendar:read";
/// Currency-rate read scope.
pub const CURRENCY_READ: &str = "ibkr:currency:read";
/// Transfer-history read scope.
pub const TRANSFERS_READ: &str = "ibkr:transfers:read";
/// Approval creation scope.
pub const APPROVALS_CREATE: &str = "ibkr:approvals:create";

/// All read scopes allowed by default.
pub const READ_SCOPES: &[&str] = &[
    HEALTH_READ,
    ACCOUNTS_READ,
    PORTFOLIO_READ,
    POSITIONS_READ,
    MARKETDATA_READ,
    ORDERS_READ,
    AUDIT_READ,
    AUDIT_EXPORT,
    RISK_READ,
    OPTIONS_READ,
    MARKETDATA_DEPTH_READ,
    SCANNER_READ,
    NEWS_READ,
    FUNDAMENTALS_READ,
    CALENDAR_READ,
    CURRENCY_READ,
    TRANSFERS_READ,
];

/// Write-adjacent preview scopes.
pub const PREVIEW_SCOPES: &[&str] = &[ORDERS_PREVIEW, RISK_READ];

/// Paper trading scopes.
pub const PAPER_SCOPES: &[&str] = &[
    ORDERS_PAPER_SUBMIT,
    ORDERS_PAPER_CANCEL,
    ORDERS_PAPER_MODIFY,
    APPROVALS_CREATE,
];

/// Live trading scopes.
pub const LIVE_SCOPES: &[&str] = &[ORDERS_LIVE_SUBMIT, ORDERS_LIVE_CANCEL, ORDERS_LIVE_MODIFY];

/// All local scopes known by the gateway.
pub const LOCAL_SCOPES: &[&str] = &[
    HEALTH_READ,
    ACCOUNTS_READ,
    PORTFOLIO_READ,
    POSITIONS_READ,
    MARKETDATA_READ,
    ORDERS_READ,
    AUDIT_READ,
    AUDIT_EXPORT,
    ORDERS_PREVIEW,
    RISK_READ,
    ORDERS_PAPER_SUBMIT,
    ORDERS_PAPER_CANCEL,
    ORDERS_PAPER_MODIFY,
    ORDERS_LIVE_SUBMIT,
    ORDERS_LIVE_CANCEL,
    ORDERS_LIVE_MODIFY,
    OPTIONS_READ,
    MARKETDATA_DEPTH_READ,
    SCANNER_READ,
    NEWS_READ,
    FUNDAMENTALS_READ,
    CALENDAR_READ,
    CURRENCY_READ,
    TRANSFERS_READ,
    APPROVALS_CREATE,
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
                format!("Scope is not allowed in a read-only scope set: {scope}"),
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
        Self::local_matching(scopes, is_preview_scope_set)
    }

    /// Creates a local scope set that may include paper scopes.
    pub fn local_with_paper(
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, GatewayError> {
        Self::local_matching(scopes, is_paper_scope_set)
    }

    /// Creates a local scope set that may include live trading scopes.
    pub fn local_with_live(
        scopes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, GatewayError> {
        Self::local(scopes)
    }

    fn local(scopes: impl IntoIterator<Item = impl Into<String>>) -> Result<Self, GatewayError> {
        Self::local_matching(scopes, is_local_scope)
    }

    fn local_matching(
        scopes: impl IntoIterator<Item = impl Into<String>>,
        allowed: fn(&str) -> bool,
    ) -> Result<Self, GatewayError> {
        let scopes = scopes
            .into_iter()
            .map(Into::into)
            .collect::<BTreeSet<String>>();

        if let Some(scope) = scopes.iter().find(|scope| !allowed(scope)) {
            return Err(GatewayError::new(
                ErrorCode::AuthScopeNotAllowedInMvp,
                format!("Scope is not allowed locally: {scope}"),
                false,
                Some("Remove scopes outside the selected local capability tier".to_string()),
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

/// Returns true when the scope is a read-only scope.
#[must_use]
pub fn is_read_scope(scope: &str) -> bool {
    READ_SCOPES.contains(&scope)
}

/// Returns true when the scope is known by the local gateway.
#[must_use]
pub fn is_local_scope(scope: &str) -> bool {
    LOCAL_SCOPES.contains(&scope)
}

fn is_preview_scope_set(scope: &str) -> bool {
    is_read_scope(scope) || PREVIEW_SCOPES.contains(&scope)
}

fn is_paper_scope_set(scope: &str) -> bool {
    is_preview_scope_set(scope) || PAPER_SCOPES.contains(&scope)
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
