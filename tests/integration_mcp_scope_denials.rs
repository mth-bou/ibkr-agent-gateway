use ibkr_auth::{HEALTH_READ, ScopeSet};

#[test]
fn mcp_scope_guard_denies_missing_scope() -> Result<(), Box<dyn std::error::Error>> {
    let scopes = ScopeSet::read_only([HEALTH_READ])?;
    let result = ibkr_mcp::enforce_scope(&scopes, "ibkr:accounts:read");
    assert!(result.is_err());
    Ok(())
}
