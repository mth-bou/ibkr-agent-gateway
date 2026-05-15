use ibkr_auth::{ORDERS_LIVE_CANCEL, ORDERS_LIVE_SUBMIT};

#[test]
fn live_tools_are_absent_from_default_discovery() {
    let names = ibkr_mcp::broker_tool_schemas()
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();

    assert!(!names.contains(&"ibkr_live_order_submit".to_string()));
    assert!(!names.contains(&"ibkr_live_order_cancel".to_string()));
}

#[test]
fn live_tools_are_discoverable_only_when_enabled() -> Result<(), Box<dyn std::error::Error>> {
    let tools = ibkr_mcp::broker_tool_schemas_with_live(true);

    let submit = tools
        .iter()
        .find(|tool| tool.name == "ibkr_live_order_submit")
        .ok_or("live submit tool should be present when enabled")?;
    assert_eq!(submit.scope, ORDERS_LIVE_SUBMIT);

    let cancel = tools
        .iter()
        .find(|tool| tool.name == "ibkr_live_order_cancel")
        .ok_or("live cancel tool should be present when enabled")?;
    assert_eq!(cancel.scope, ORDERS_LIVE_CANCEL);
    Ok(())
}
