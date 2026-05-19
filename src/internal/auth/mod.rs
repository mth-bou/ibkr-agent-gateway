//! Local scope and authorization facade for gateway tools.

pub mod local_user;
pub mod oauth_context;
pub mod scopes;

pub use local_user::{AuthContext, AuthContextSource, LocalUser};
pub use oauth_context::{OAuthContextInput, RemoteAuthContext, remote_auth_context_from_input};
pub use scopes::{
    ACCOUNTS_READ, APPROVALS_CREATE, AUDIT_EXPORT, AUDIT_READ, CALENDAR_READ, CURRENCY_READ,
    FUNDAMENTALS_READ, HEALTH_READ, LIVE_SCOPES, LOCAL_SCOPES, MARKETDATA_DEPTH_READ,
    MARKETDATA_READ, NEWS_READ, OPTIONS_READ, ORDERS_LIVE_CANCEL, ORDERS_LIVE_MODIFY,
    ORDERS_LIVE_SUBMIT, ORDERS_PAPER_CANCEL, ORDERS_PAPER_MODIFY, ORDERS_PAPER_SUBMIT,
    ORDERS_PREVIEW, ORDERS_READ, PAPER_SCOPES, PORTFOLIO_READ, POSITIONS_READ, PREVIEW_SCOPES,
    READ_SCOPES, RISK_READ, SCANNER_READ, ScopeSet, TRANSFERS_READ, is_local_scope, is_read_scope,
    require_scope,
};

#[cfg(test)]
mod tests {
    use super::{HEALTH_READ, ScopeSet, is_read_scope, require_scope};

    #[test]
    fn accepts_read_scope() {
        assert!(is_read_scope(HEALTH_READ));
    }

    #[test]
    fn rejects_write_scope() {
        assert!(ScopeSet::read_only(["ibkr:orders:submit"]).is_err());
    }

    #[test]
    fn accepts_preview_scope_with_preview_constructor() {
        assert!(ScopeSet::local_with_preview([super::ORDERS_PREVIEW]).is_ok());
        assert!(ScopeSet::read_only([super::ORDERS_PREVIEW]).is_err());
    }

    #[test]
    fn accepts_paper_scope_with_paper_constructor() {
        assert!(ScopeSet::local_with_paper([super::ORDERS_PAPER_SUBMIT]).is_ok());
        assert!(ScopeSet::read_only([super::ORDERS_PAPER_SUBMIT]).is_err());
    }

    #[test]
    fn denies_missing_scope() {
        let scopes = ScopeSet::read_only([HEALTH_READ]);
        let Ok(scopes) = scopes else {
            unreachable!("read scope should be accepted");
        };

        assert!(require_scope(&scopes, "ibkr:accounts:read").is_err());
    }
}
