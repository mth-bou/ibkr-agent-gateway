#[tokio::test]
async fn cli_audit_tail_command_runs() -> Result<(), Box<dyn std::error::Error>> {
    ibkr_cli::run_from_args(["ibkr-agent", "audit", "tail", "--limit", "10", "--json"]).await?;
    Ok(())
}
