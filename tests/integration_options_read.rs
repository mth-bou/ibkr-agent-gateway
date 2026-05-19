#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    backend::{FakeBackend, FakeFixtureStore, IbkrBackend},
    domain::ContractId,
};

#[tokio::test]
async fn fake_backend_returns_options_chain_and_greeks() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));

    let chain = backend.options_chain("AAPL").await?;
    assert_eq!(chain.underlying_symbol, "AAPL");
    assert_eq!(chain.entries.len(), 2);
    assert_eq!(
        chain.entries[0].right,
        ibkr_agent_gateway::testing::domain::OptionRight::Call
    );

    let greeks = backend
        .option_greeks(&ContractId::from_static("700001"))
        .await?;
    assert_eq!(greeks.contract_id.as_str(), "700001");
    assert!(greeks.delta.is_some());
    Ok(())
}
