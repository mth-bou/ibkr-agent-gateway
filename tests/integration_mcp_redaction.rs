use ibkr_mcp::broker_tool_schemas;

#[test]
fn mcp_output_schemas_advertise_redaction_boundary() {
    for tool in broker_tool_schemas() {
        assert_eq!(
            tool.output_schema["x-redaction"],
            "tokens,cookies,credentials,headers,local_paths"
        );
    }
}
