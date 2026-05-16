use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_agent_gateway::testing::domain::BrokerSessionVisibility;

#[tokio::test]
async fn fake_backend_keepalive_returns_usable_status() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let status = backend.keepalive().await?;

    assert_eq!(status.status, BrokerSessionVisibility::Usable);
    assert!(status.last_keepalive_at.is_some());
    Ok(())
}
