use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_agent_gateway::testing::domain::{AccountId, OrdersHistoryRequest, ReadOnlyOrderStatus};

#[tokio::test]
async fn fake_backend_returns_bounded_order_history() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let Some(account_id) = AccountId::new("DU1234567") else {
        return Err("account id rejected".into());
    };

    let history = backend
        .orders_history(&OrdersHistoryRequest {
            account_id: account_id.clone(),
            from: None,
            to: None,
            status: None,
            limit: 100,
        })
        .await?;

    assert_eq!(history.account_id, account_id);
    assert_eq!(history.orders.len(), 2);
    assert!(
        history
            .orders
            .iter()
            .any(|order| order.status == ReadOnlyOrderStatus::Filled)
    );
    assert!(
        history
            .orders
            .iter()
            .any(|order| order.status == ReadOnlyOrderStatus::Cancelled)
    );
    Ok(())
}
