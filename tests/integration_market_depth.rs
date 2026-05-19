#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    backend::{FakeBackend, FakeFixtureStore, IbkrBackend},
    domain::ContractId,
};

#[tokio::test]
async fn fake_backend_returns_market_depth() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));

    let depth = backend
        .market_depth(&ContractId::from_static("265598"))
        .await?;
    assert_eq!(depth.contract_id.as_str(), "265598");
    assert_eq!(depth.bids.len(), 1);
    assert_eq!(depth.asks.len(), 1);
    Ok(())
}
