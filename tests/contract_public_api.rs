use ibkr_agent_gateway::prelude::*;

#[tokio::test]
async fn public_gateway_fake_backend_session_and_accounts_work() -> Result<(), GatewayError> {
    let gateway = Gateway::new(GatewayConfig::fake_local())?;

    let status = gateway.session_status().await?;
    assert_eq!(status.backend, BrokerBackendKind::Fake);
    assert_eq!(status.status, BrokerSessionVisibility::Usable);

    let accounts = gateway.list_accounts().await?;
    assert!(!accounts.is_empty());

    let contracts = gateway.search_contracts("AAPL").await?;
    assert!(contracts.iter().any(|contract| contract.is_unique_match));

    Ok(())
}
