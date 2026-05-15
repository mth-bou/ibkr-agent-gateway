#[path = "common/live.rs"]
mod live;

use ibkr_domain::{BrokerOrderId, ErrorCode, LocalUserId};
use ibkr_orders::{
    IdempotencyKey, IdempotencyStore, KillSwitch, LiveCancelRequest, PaperToLiveMigrationChecklist,
    cancel_live_order, submit_live_order,
};

#[test]
fn live_submit_replays_same_request_and_rejects_conflicts() -> Result<(), Box<dyn std::error::Error>>
{
    let request = live::live_submit_request()?;
    let mut idempotency_store = IdempotencyStore::default();

    let first = submit_live_order(request.clone(), &mut idempotency_store)?;
    let replayed = submit_live_order(request.clone(), &mut idempotency_store)?;
    assert_eq!(first.idempotency_key, replayed.idempotency_key);
    assert_eq!(
        first.lifecycle.broker_order_id,
        replayed.lifecycle.broker_order_id
    );

    let mut conflict = request;
    conflict.order = live::validated_order(live::account_id())?;
    let Err(error) = submit_live_order(conflict, &mut idempotency_store) else {
        return Err("conflicting live submit idempotency key should be rejected".into());
    };
    assert_eq!(error.code, ErrorCode::PaperIdempotencyConflict);
    Ok(())
}

#[test]
fn live_cancel_replays_same_request_and_rejects_conflicts() -> Result<(), Box<dyn std::error::Error>>
{
    let account_id = live::account_id();
    let request = LiveCancelRequest {
        account_id: account_id.clone(),
        broker_order_id: BrokerOrderId::from_static("live-order-local"),
        idempotency_key: IdempotencyKey::new("live-cancel-key")?,
        live_config: live::live_config(account_id),
        live_scope_granted: true,
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "test open"),
        audit_available: true,
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    };
    let mut idempotency_store = IdempotencyStore::default();

    let first = cancel_live_order(request.clone(), &mut idempotency_store)?;
    let replayed = cancel_live_order(request.clone(), &mut idempotency_store)?;
    assert_eq!(first.idempotency_key, replayed.idempotency_key);
    assert_eq!(
        first.lifecycle.broker_order_id,
        replayed.lifecycle.broker_order_id
    );

    let mut conflict = request;
    conflict.broker_order_id = BrokerOrderId::from_static("different-live-order");
    let Err(error) = cancel_live_order(conflict, &mut idempotency_store) else {
        return Err("conflicting live cancel idempotency key should be rejected".into());
    };
    assert_eq!(error.code, ErrorCode::PaperIdempotencyConflict);
    Ok(())
}
