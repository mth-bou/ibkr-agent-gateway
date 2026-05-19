use ibkr_agent_gateway::testing::mcp::{
    FORBIDDEN_TOOL_NAMES, broker_tool_schemas, local_tool_schemas,
};

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
            "ibkr_session_renew",
            "ibkr_kill_switch_status",
            "ibkr_accounts_list",
            "ibkr_account_metadata",
            "ibkr_account_summary",
            "ibkr_pnl_daily",
            "ibkr_pnl_realtime",
            "ibkr_positions_list",
            "ibkr_portfolio_snapshot",
            "ibkr_contracts_search",
            "ibkr_contract_resolve",
            "ibkr_market_snapshot",
            "ibkr_historical_bars",
            "ibkr_options_chain",
            "ibkr_option_greeks",
            "ibkr_market_depth",
            "ibkr_scanner_run",
            "ibkr_orders_list",
            "ibkr_orders_history",
            "ibkr_order_status",
            "ibkr_executions_list",
            "ibkr_limits_status",
            "ibkr_audit_tail",
            "ibkr_audit_export",
        ]
    );

    for forbidden in FORBIDDEN_TOOL_NAMES {
        assert!(!names.contains(forbidden));
    }
}

#[test]
fn mcp_local_tool_list_contains_current_maturity_baseline() {
    let tools = local_tool_schemas();
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
            "ibkr_session_renew",
            "ibkr_kill_switch_status",
            "ibkr_accounts_list",
            "ibkr_account_metadata",
            "ibkr_account_summary",
            "ibkr_pnl_daily",
            "ibkr_pnl_realtime",
            "ibkr_positions_list",
            "ibkr_portfolio_snapshot",
            "ibkr_contracts_search",
            "ibkr_contract_resolve",
            "ibkr_market_snapshot",
            "ibkr_historical_bars",
            "ibkr_options_chain",
            "ibkr_option_greeks",
            "ibkr_market_depth",
            "ibkr_scanner_run",
            "ibkr_orders_list",
            "ibkr_orders_history",
            "ibkr_order_status",
            "ibkr_executions_list",
            "ibkr_limits_status",
            "ibkr_audit_tail",
            "ibkr_audit_export",
            "ibkr_order_preview",
            "ibkr_bracket_order_preview",
            "ibkr_paper_order_submit",
            "ibkr_paper_order_cancel",
            "ibkr_paper_order_modify",
            "ibkr_paper_bracket_order_submit",
            "ibkr_live_order_submit",
            "ibkr_live_order_cancel",
            "ibkr_live_order_modify",
            "ibkr_live_bracket_order_submit",
        ]
    );

    for forbidden in FORBIDDEN_TOOL_NAMES {
        assert!(!names.contains(forbidden));
    }
    assert!(FORBIDDEN_TOOL_NAMES.contains(&"ibkr_order_modify"));
}
