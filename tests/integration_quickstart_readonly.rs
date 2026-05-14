#[tokio::test]
async fn quickstart_readonly_commands_run() -> Result<(), Box<dyn std::error::Error>> {
    let commands = [
        vec!["ibkr-agent", "health", "--json"],
        vec!["ibkr-agent", "backend", "status", "--json"],
        vec!["ibkr-agent", "session", "requirements", "--json"],
        vec!["ibkr-agent", "accounts", "list", "--json"],
        vec![
            "ibkr-agent",
            "account",
            "summary",
            "--account",
            "DU1234567",
            "--json",
        ],
        vec![
            "ibkr-agent",
            "portfolio",
            "snapshot",
            "--account",
            "DU1234567",
            "--json",
        ],
        vec![
            "ibkr-agent",
            "positions",
            "list",
            "--account",
            "DU1234567",
            "--json",
        ],
        vec![
            "ibkr-agent",
            "contracts",
            "search",
            "AAPL",
            "--asset-class",
            "stock",
            "--currency",
            "USD",
            "--exchange",
            "SMART",
            "--json",
        ],
        vec![
            "ibkr-agent",
            "market",
            "snapshot",
            "--contract-id",
            "265598",
            "--json",
        ],
        vec![
            "ibkr-agent",
            "orders",
            "list",
            "--account",
            "DU1234567",
            "--json",
        ],
        vec![
            "ibkr-agent",
            "executions",
            "list",
            "--account",
            "DU1234567",
            "--json",
        ],
        vec![
            "ibkr-agent",
            "mcp",
            "serve",
            "--transport",
            "stdio",
            "--json",
        ],
        vec!["ibkr-agent", "audit", "tail", "--limit", "20", "--json"],
    ];

    for command in commands {
        ibkr_cli::run_from_args(command).await?;
    }

    let write_result = ibkr_cli::run_from_args(["ibkr-agent", "orders", "submit"]).await;
    assert!(write_result.is_err());
    Ok(())
}
