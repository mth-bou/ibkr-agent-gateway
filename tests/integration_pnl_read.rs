use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_agent_gateway::testing::domain::{AccountId, MarketDataStatus};

#[tokio::test]
async fn fake_backend_returns_daily_and_realtime_pnl() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let Some(account_id) = AccountId::new("DU1234567") else {
        return Err("account id rejected".into());
    };

    let daily = backend.pnl_daily(&account_id).await?;
    let realtime = backend.pnl_realtime(&account_id).await?;

    assert_eq!(daily.account_id, account_id);
    assert_eq!(daily.data_status, MarketDataStatus::Live);
    assert_eq!(daily.total_pnl.currency.as_str(), "USD");
    assert_eq!(realtime.rows.len(), 1);
    assert_eq!(realtime.rows[0].symbol.as_deref(), Some("AAPL"));
    Ok(())
}
