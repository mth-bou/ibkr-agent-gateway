use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore};
use ibkr_agent_gateway::testing::domain::BrokerSessionVisibility;

#[tokio::test]
async fn mcp_keepalive_tick_uses_backend_keepalive() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let status = ibkr_agent_gateway::testing::mcp::keepalive_once(&backend).await?;

    assert_eq!(status.status, BrokerSessionVisibility::Usable);
    Ok(())
}
