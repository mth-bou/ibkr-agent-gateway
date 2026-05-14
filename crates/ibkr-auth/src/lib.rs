//! Local scope and authorization facade for gateway tools.

pub mod local_user;
pub mod scopes;

pub use local_user::{AuthContext, AuthContextSource, LocalUser};
pub use scopes::{
    ACCOUNTS_READ, AUDIT_READ, HEALTH_READ, MARKETDATA_READ, ORDERS_READ, PORTFOLIO_READ,
    POSITIONS_READ, READ_SCOPES, ScopeSet, is_read_scope, require_scope,
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
    fn denies_missing_scope() {
        let scopes = ScopeSet::read_only([HEALTH_READ]);
        let Ok(scopes) = scopes else {
            unreachable!("read scope should be accepted");
        };

        assert!(require_scope(&scopes, "ibkr:accounts:read").is_err());
    }
}
