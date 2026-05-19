#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    backend::{FakeBackend, FakeFixtureStore, IbkrBackend},
    domain::AccountId,
};

#[tokio::test]
async fn fake_backend_returns_transfer_history() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));

    let history = backend
        .transfer_history(&AccountId::from_static("DU1234567"))
        .await?;
    assert_eq!(history.account_id.as_str(), "DU1234567");
    assert_eq!(history.transfers[0].transfer_id, "transfer-1");
    Ok(())
}
