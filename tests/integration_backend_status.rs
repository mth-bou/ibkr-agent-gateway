use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_agent_gateway::testing::domain::{BrokerBackendKind, BrokerSessionVisibility};

#[tokio::test]
async fn fake_backend_returns_usable_status() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let status = backend.session_status().await?;

    assert_eq!(status.backend, BrokerBackendKind::Fake);
    assert_eq!(status.status, BrokerSessionVisibility::Usable);
    assert!(status.user_action.is_none());
    Ok(())
}
