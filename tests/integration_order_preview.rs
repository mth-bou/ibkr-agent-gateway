#[tokio::test]
async fn cli_order_preview_requires_explicit_enablement() {
    let result = ibkr_cli::run_from_args([
        "ibkr-agent",
        "orders",
        "preview",
        "--account",
        "DU1234567",
        "--symbol",
        "AAPL",
        "--side",
        "buy",
        "--quantity",
        "1",
        "--limit-price",
        "100",
        "--json",
    ])
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn cli_order_preview_creates_non_executable_preview() -> Result<(), Box<dyn std::error::Error>>
{
    ibkr_cli::run_from_args([
        "ibkr-agent",
        "orders",
        "preview",
        "--account",
        "DU1234567",
        "--symbol",
        "AAPL",
        "--side",
        "buy",
        "--quantity",
        "1",
        "--limit-price",
        "100",
        "--enable-preview",
        "--json",
    ])
    .await?;

    Ok(())
}
