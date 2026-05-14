use ibkr_mcp::{FORBIDDEN_TOOL_NAMES, broker_tool_schemas};

#[test]
fn mcp_broker_tool_list_contains_only_readonly_tools() {
    let tools = broker_tool_schemas();
    let names = tools
        .iter()
        .map(|tool| tool.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "ibkr_health",
            "ibkr_backend_status",
            "ibkr_session_requirements",
            "ibkr_accounts_list",
            "ibkr_account_summary",
            "ibkr_positions_list",
            "ibkr_portfolio_snapshot",
            "ibkr_contracts_search",
            "ibkr_contract_resolve",
            "ibkr_market_snapshot",
            "ibkr_historical_bars",
            "ibkr_orders_list",
            "ibkr_order_preview",
            "ibkr_order_status",
            "ibkr_executions_list",
            "ibkr_audit_tail",
            "ibkr_paper_order_submit",
            "ibkr_paper_order_cancel",
        ]
    );

    for forbidden in FORBIDDEN_TOOL_NAMES {
        assert!(!names.contains(forbidden));
    }
}

#[test]
fn mcp_broker_tool_list_contract_placeholder() {
    assert_eq!(ibkr_agent_gateway::HARNESS_NAME, "ibkr-agent-gateway");
}
