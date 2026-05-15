use ibkr_mcp::broker_tool_schemas;
use ibkr_provider_compat::generic_mcp::schema_snapshots;

const EXPECTED_REDACTION: &str = "tokens,cookies,credentials,headers,local_paths";
const CLIENT_EXAMPLES: &[&str] = &[
    "examples/mcp-clients/continue.json",
    "examples/mcp-clients/cursor.json",
    "examples/mcp-clients/generic-inspector.json",
];
const SECRET_MARKERS: &[&str] = &[
    "bearer ",
    "client_secret",
    "cookie",
    "ibkr_password",
    "refresh_token",
    "secret",
    "session_token",
];

#[test]
fn provider_visible_tool_schemas_keep_redaction_boundary() {
    for tool in broker_tool_schemas() {
        assert_eq!(tool.output_schema["x-redaction"], EXPECTED_REDACTION);
    }

    for snapshot in schema_snapshots() {
        assert_eq!(snapshot.redaction_report.secret_scan_result, "pass");
        assert!(snapshot.redaction_report.leaks_found.is_empty());
    }
}

#[test]
fn provider_client_examples_do_not_embed_secrets() -> Result<(), Box<dyn std::error::Error>> {
    for example_path in CLIENT_EXAMPLES {
        let contents = std::fs::read_to_string(example_path)?;
        let lower_contents = contents.to_lowercase();

        for marker in SECRET_MARKERS {
            assert!(
                !lower_contents.contains(marker),
                "{example_path} contains secret-like marker {marker}"
            );
        }
    }

    Ok(())
}
