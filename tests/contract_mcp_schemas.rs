use ibkr_agent_gateway::testing::mcp::broker_tool_schemas;

#[test]
fn every_mcp_tool_has_scope_and_object_schemas() {
    for tool in broker_tool_schemas() {
        assert!(!tool.scope.is_empty());
        assert_eq!(tool.input_schema["type"], "object");
        assert_eq!(tool.output_schema["type"], "object");
    }
}

#[test]
fn mcp_tool_schemas_match_required_scope_contract() {
    let tools = broker_tool_schemas()
        .into_iter()
        .map(|tool| (tool.name, tool.scope, tool.input_schema["required"].clone()))
        .collect::<Vec<_>>();

    assert!(tools.contains(&(
        "ibkr_health".to_string(),
        "ibkr:health:read".to_string(),
        serde_json::json!([])
    )));
    assert!(tools.contains(&(
        "ibkr_accounts_list".to_string(),
        "ibkr:accounts:read".to_string(),
        serde_json::json!([])
    )));
    assert!(tools.contains(&(
        "ibkr_account_summary".to_string(),
        "ibkr:portfolio:read".to_string(),
        serde_json::json!(["account_id"])
    )));
    assert!(tools.contains(&(
        "ibkr_positions_list".to_string(),
        "ibkr:positions:read".to_string(),
        serde_json::json!(["account_id"])
    )));
    assert!(tools.contains(&(
        "ibkr_market_snapshot".to_string(),
        "ibkr:marketdata:read".to_string(),
        serde_json::json!(["contract_id"])
    )));
    assert!(tools.contains(&(
        "ibkr_order_status".to_string(),
        "ibkr:orders:read".to_string(),
        serde_json::json!(["account_id", "broker_order_id"])
    )));
    assert!(tools.contains(&(
        "ibkr_audit_tail".to_string(),
        "ibkr:audit:read".to_string(),
        serde_json::json!(["limit"])
    )));
}
