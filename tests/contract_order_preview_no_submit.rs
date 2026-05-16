use ibkr_domain::ErrorCode;

#[test]
fn submit_cancel_and_approve_tools_remain_forbidden_in_mcp() {
    for name in [
        "ibkr_order_submit",
        "ibkr_order_cancel",
        "ibkr_order_approve",
    ] {
        let error = ibkr_mcp::refuse_forbidden_tool(name);
        assert_eq!(error.code, ErrorCode::ReadonlyWriteForbidden);
        assert!(ibkr_mcp::is_forbidden_tool_name(name));
    }
}

#[test]
fn submit_cancel_and_approve_cli_paths_remain_refused() {
    for action in ["submit", "cancel", "approve"] {
        let error = ibkr_agent_gateway::cli::commands::orders::refuse_write(action);
        assert!(error.is_err());
    }
}
