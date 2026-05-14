#[tokio::test]
async fn cli_health_backend_session_and_accounts_commands_run()
-> Result<(), Box<dyn std::error::Error>> {
    ibkr_cli::run_from_args(["ibkr-agent", "health", "--json"]).await?;
    ibkr_cli::run_from_args(["ibkr-agent", "backend", "status", "--json"]).await?;
    ibkr_cli::run_from_args(["ibkr-agent", "session", "requirements", "--json"]).await?;
    ibkr_cli::run_from_args(["ibkr-agent", "accounts", "list", "--json"]).await?;
    ibkr_cli::run_from_args([
        "ibkr-agent",
        "mcp",
        "serve",
        "--transport",
        "stdio",
        "--json",
    ])
    .await?;
    Ok(())
}

#[test]
fn cli_us1_contract_placeholder() {
    assert_eq!(ibkr_agent_gateway::HARNESS_NAME, "ibkr-agent-gateway");
}
