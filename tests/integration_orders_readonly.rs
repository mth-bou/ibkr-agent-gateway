use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_agent_gateway::testing::domain::AccountId;

#[tokio::test]
async fn fake_backend_returns_readonly_orders_and_executions()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let Some(account_id) = AccountId::new("DU1234567") else {
        return Err("account id rejected".into());
    };

    let orders = backend.orders(&account_id).await?;
    let status = backend.order_status(&account_id, "123").await?;
    let executions = backend.executions(&account_id).await?;

    assert_eq!(orders.len(), 1);
    assert_eq!(status.broker_order_id.as_str(), "123");
    assert_eq!(executions.len(), 1);
    Ok(())
}
