use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_agent_gateway::testing::domain::AccountId;

#[tokio::test]
async fn fake_backend_returns_portfolio_and_positions() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let Some(account_id) = AccountId::new("DU1234567") else {
        return Err("account id rejected".into());
    };

    let summary = backend.account_summary(&account_id).await?;
    let positions = backend.positions(&account_id).await?;

    assert_eq!(summary["account_id"], "DU1234567");
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0]["symbol"], "AAPL");
    Ok(())
}
