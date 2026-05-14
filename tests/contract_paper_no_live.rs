#[test]
fn live_order_tools_are_not_discoverable() {
    let tools = ibkr_mcp::broker_tool_schemas();
    let names = tools
        .iter()
        .map(|tool| tool.name.as_str())
        .collect::<Vec<_>>();

    for forbidden in [
        "ibkr_live_order_submit",
        "ibkr_live_order_cancel",
        "ibkr_order_submit",
        "ibkr_order_cancel",
        "ibkr_order_approve",
    ] {
        assert!(!names.contains(&forbidden));
    }
}

#[test]
fn generic_submit_cancel_and_approve_remain_refused() {
    for name in [
        "ibkr_order_submit",
        "ibkr_order_cancel",
        "ibkr_order_approve",
    ] {
        assert!(ibkr_mcp::is_forbidden_tool_name(name));
    }
}
