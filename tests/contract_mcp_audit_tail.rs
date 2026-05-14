use ibkr_mcp::broker_tool_schemas;

#[test]
fn mcp_audit_tail_is_exposed_after_us4_with_audit_scope() {
    let tools = broker_tool_schemas();
    let mut found = false;

    for tool in tools {
        if tool.name == "ibkr_audit_tail" {
            found = true;
            assert_eq!(tool.scope, "ibkr:audit:read");
            assert_eq!(tool.input_schema["required"], serde_json::json!(["limit"]));
        }
    }

    assert!(found);
}
