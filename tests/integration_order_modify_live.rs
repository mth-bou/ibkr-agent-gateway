#![cfg(feature = "unstable-internal-test-support")]

#[path = "common/live.rs"]
mod live;

use ibkr_agent_gateway::testing::{
    domain::{BrokerOrderId, ErrorCode, LocalUserId, Money, Quantity},
    orders::{
        IdempotencyKey, IdempotencyStore, KillSwitch, LiveModifyRequest, LiveOrderLifecycleStatus,
        LocalCandidateLiveWriter, OrderModifyFields, modify_live_order,
    },
    risk::StaticPolicyRegistry,
};
use rust_decimal::Decimal;

#[tokio::test]
async fn live_modify_runs_gates_and_returns_open_lifecycle()
-> Result<(), Box<dyn std::error::Error>> {
    let account = live::account_id();
    let Some(currency) = ibkr_agent_gateway::testing::domain::CurrencyCode::new("USD") else {
        return Err("USD should be valid".into());
    };
    let mut approved_order = live::validated_order(account.clone())?;
    approved_order.quantity = Quantity::new(Decimal::new(2, 0));
    approved_order.limit_price = Some(Money {
        amount: Decimal::new(101, 0),
        currency: currency.clone(),
    });
    let approval = live::approval_for_order(account.clone(), &approved_order);
    let request = LiveModifyRequest {
        account_id: account.clone(),
        broker_order_id: BrokerOrderId::from_static("live-order-1"),
        changes: OrderModifyFields {
            quantity: Some(Quantity::new(Decimal::new(2, 0))),
            limit_price: Some(Money {
                amount: Decimal::new(101, 0),
                currency,
            }),
            stop_price: None,
            time_in_force: None,
            trailing_amount: None,
            trailing_percent: None,
        },
        approved_order,
        approval,
        idempotency_key: IdempotencyKey::new("live-modify-service-1")?,
        live_config: live::live_config(account.clone()),
        live_scope_granted: true,
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "test"),
        audit_available: true,
        live_limit_context: live::live_limit_context()?,
        migration_checklist: live::live_submit_request()?.migration_checklist,
    };
    let mut idempotency_store = IdempotencyStore::default();
    let policy_registry = StaticPolicyRegistry::single(live::live_limit_policy()?);

    let result = modify_live_order(
        request,
        &LocalCandidateLiveWriter,
        &policy_registry,
        &mut idempotency_store,
    )
    .await?;

    assert_eq!(result.lifecycle.account_id, account);
    assert_eq!(result.lifecycle.broker_order_id.as_str(), "live-order-1");
    assert_eq!(result.lifecycle.status, LiveOrderLifecycleStatus::Open);
    Ok(())
}

#[tokio::test]
async fn live_modify_refuses_closed_kill_switch() -> Result<(), Box<dyn std::error::Error>> {
    let account = live::account_id();
    let mut approved_order = live::validated_order(account.clone())?;
    approved_order.quantity = Quantity::new(Decimal::new(2, 0));
    let approval = live::approval_for_order(account.clone(), &approved_order);
    let request = LiveModifyRequest {
        account_id: account.clone(),
        broker_order_id: BrokerOrderId::from_static("live-order-1"),
        changes: OrderModifyFields {
            quantity: Some(Quantity::new(Decimal::new(2, 0))),
            limit_price: None,
            stop_price: None,
            time_in_force: None,
            trailing_amount: None,
            trailing_percent: None,
        },
        approved_order,
        approval,
        idempotency_key: IdempotencyKey::new("live-modify-service-closed")?,
        live_config: live::live_config(account),
        live_scope_granted: true,
        kill_switch: KillSwitch::closed(LocalUserId::from_static("operator"), "test"),
        audit_available: true,
        live_limit_context: live::live_limit_context()?,
        migration_checklist: live::live_submit_request()?.migration_checklist,
    };
    let mut idempotency_store = IdempotencyStore::default();
    let policy_registry = StaticPolicyRegistry::single(live::live_limit_policy()?);

    let error = modify_live_order(
        request,
        &LocalCandidateLiveWriter,
        &policy_registry,
        &mut idempotency_store,
    )
    .await
    .err()
    .ok_or("closed kill switch should refuse")?;

    assert_eq!(error.code, ErrorCode::LiveKillSwitchClosed);
    Ok(())
}
