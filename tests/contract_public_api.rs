use ibkr_agent_gateway::prelude::*;
use url::Url;

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

#[test]
fn public_gateway_rejects_remote_tls_bypass() -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse("https://broker.example.com/v1/api")?;
    let Err(error) = Gateway::new(GatewayConfig::client_portal(url).with_verify_tls(false)) else {
        unreachable!("TLS bypass must be limited to localhost Client Portal Gateway URLs");
    };

    assert_eq!(error.code, ErrorCode::ConfigTlsBypassNonLocalhost);
    Ok(())
}
