#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    config::PaperTradingConfig,
    domain::{AccountId, BrokerOrderId, ErrorCode, Money},
    orders::{
        IdempotencyKey, IdempotencyStore, LocalCandidatePaperWriter, OrderModifyFields,
        PaperModifyRequest, modify_paper_order,
    },
};
use rust_decimal::Decimal;

#[tokio::test]
async fn paper_modify_replays_same_request_and_refuses_key_conflict()
-> Result<(), Box<dyn std::error::Error>> {
    let account = AccountId::from_static("DU1234567");
    let Some(currency) = ibkr_agent_gateway::testing::domain::CurrencyCode::new("USD") else {
        return Err("USD should be valid".into());
    };
    let mut idempotency_store = IdempotencyStore::default();
    let key = IdempotencyKey::new("paper-modify-idempotent")?;
    let request = PaperModifyRequest {
        account_id: account.clone(),
        broker_order_id: BrokerOrderId::from_static("paper-order-1"),
        changes: OrderModifyFields {
            quantity: None,
            limit_price: Some(Money {
                amount: Decimal::new(12450, 2),
                currency: currency.clone(),
            }),
            stop_price: None,
            time_in_force: None,
            trailing_amount: None,
            trailing_percent: None,
        },
        idempotency_key: key.clone(),
        paper_config: PaperTradingConfig {
            enabled: true,
            allowed_accounts: vec![account.clone()],
        },
    };

    let first = modify_paper_order(
        request.clone(),
        &LocalCandidatePaperWriter,
        &mut idempotency_store,
    )
    .await?;
    let replay = modify_paper_order(
        request.clone(),
        &LocalCandidatePaperWriter,
        &mut idempotency_store,
    )
    .await?;
    assert_eq!(first.lifecycle.account_id, replay.lifecycle.account_id);
    assert_eq!(
        first.lifecycle.broker_order_id,
        replay.lifecycle.broker_order_id
    );
    assert_eq!(first.lifecycle.status, replay.lifecycle.status);

    let conflict = modify_paper_order(
        PaperModifyRequest {
            changes: OrderModifyFields {
                quantity: None,
                limit_price: Some(Money {
                    amount: Decimal::new(12500, 2),
                    currency,
                }),
                stop_price: None,
                time_in_force: None,
                trailing_amount: None,
                trailing_percent: None,
            },
            ..request
        },
        &LocalCandidatePaperWriter,
        &mut idempotency_store,
    )
    .await
    .err()
    .ok_or("changed request with same key should conflict")?;

    assert_eq!(conflict.code, ErrorCode::PaperIdempotencyConflict);
    Ok(())
}
