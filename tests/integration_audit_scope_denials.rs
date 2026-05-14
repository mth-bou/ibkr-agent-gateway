use ibkr_auth::{HEALTH_READ, ScopeSet};
use ibkr_domain::ErrorCode;

#[test]
fn audit_tail_denies_missing_audit_scope() -> Result<(), Box<dyn std::error::Error>> {
    let scopes = ScopeSet::read_only([HEALTH_READ])?;
    let error = ibkr_mcp::enforce_scope(&scopes, "ibkr:audit:read")
        .expect_err("audit tail should require ibkr:audit:read");

    assert_eq!(error.code, ErrorCode::AuthMissingScope);
    Ok(())
}
