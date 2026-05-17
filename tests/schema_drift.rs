#[test]
fn mcp_tool_schema_snapshot_matches_expected_names_and_scopes() {
    let current = ibkr_agent_gateway::testing::mcp::broker_tool_schemas()
        .into_iter()
        .map(|tool| (tool.name, tool.scope))
        .collect::<Vec<_>>();

    let expected = vec![
        ("ibkr_health".to_string(), "ibkr:health:read".to_string()),
        (
            "ibkr_backend_status".to_string(),
            "ibkr:health:read".to_string(),
        ),
        (
            "ibkr_session_requirements".to_string(),
            "ibkr:health:read".to_string(),
        ),
        (
            "ibkr_accounts_list".to_string(),
            "ibkr:accounts:read".to_string(),
        ),
        (
            "ibkr_account_summary".to_string(),
            "ibkr:portfolio:read".to_string(),
        ),
        (
            "ibkr_positions_list".to_string(),
            "ibkr:positions:read".to_string(),
        ),
        (
            "ibkr_portfolio_snapshot".to_string(),
            "ibkr:portfolio:read".to_string(),
        ),
        (
            "ibkr_contracts_search".to_string(),
            "ibkr:marketdata:read".to_string(),
        ),
        (
            "ibkr_contract_resolve".to_string(),
            "ibkr:marketdata:read".to_string(),
        ),
        (
            "ibkr_market_snapshot".to_string(),
            "ibkr:marketdata:read".to_string(),
        ),
        (
            "ibkr_historical_bars".to_string(),
            "ibkr:marketdata:read".to_string(),
        ),
        (
            "ibkr_orders_list".to_string(),
            "ibkr:orders:read".to_string(),
        ),
        (
            "ibkr_order_status".to_string(),
            "ibkr:orders:read".to_string(),
        ),
        (
            "ibkr_executions_list".to_string(),
            "ibkr:orders:read".to_string(),
        ),
        ("ibkr_audit_tail".to_string(), "ibkr:audit:read".to_string()),
    ];

    assert_eq!(current, expected);
}

#[test]
fn mcp_tool_schema_snapshot_keeps_redaction_extension() {
    for tool in ibkr_agent_gateway::testing::mcp::broker_tool_schemas() {
        assert_eq!(
            tool.output_schema["x-redaction"],
            "tokens,cookies,credentials,headers,local_paths"
        );
    }
}
