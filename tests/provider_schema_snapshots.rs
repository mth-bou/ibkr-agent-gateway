use ibkr_agent_gateway::testing::provider_compat::generic_mcp::schema_snapshots;

#[test]
fn provider_schema_snapshots_are_stable_and_redacted() {
    let snapshots = schema_snapshots();

    assert!(!snapshots.is_empty());
    assert!(snapshots.iter().any(|snapshot| {
        snapshot.snapshot_id == "schema-ibkr_accounts_list"
            && snapshot.scenario_id == "generic-mcp-ibkr_accounts_list"
    }));

    for snapshot in snapshots {
        assert_eq!(snapshot.tool_schema_hash.len(), 64);
        assert_eq!(snapshot.response_hash.len(), 64);
        assert_eq!(snapshot.redaction_report.secret_scan_result, "pass");
        assert!(snapshot.redaction_report.leaks_found.is_empty());
    }
}
