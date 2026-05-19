use ibkr_agent_gateway::testing::auth::{HEALTH_READ, RISK_READ, ScopeSet};
use ibkr_agent_gateway::testing::mcp::local_tool_schemas_for_scopes;

#[test]
fn maturity_safety_tools_are_scope_filtered() -> Result<(), Box<dyn std::error::Error>> {
    let health_scopes = ScopeSet::read_only([HEALTH_READ])?;
    let health_tools = local_tool_schemas_for_scopes(&health_scopes)
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();

    assert!(health_tools.contains(&"ibkr_session_renew".to_string()));
    assert!(health_tools.contains(&"ibkr_kill_switch_status".to_string()));
    assert!(!health_tools.contains(&"ibkr_limits_status".to_string()));

    let risk_scopes = ScopeSet::read_only([RISK_READ])?;
    let risk_tools = local_tool_schemas_for_scopes(&risk_scopes)
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();

    assert!(risk_tools.contains(&"ibkr_limits_status".to_string()));
    assert!(!risk_tools.contains(&"ibkr_kill_switch_status".to_string()));
    Ok(())
}
