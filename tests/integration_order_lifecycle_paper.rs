use ibkr_agent_gateway::testing::approval::{ApprovalId, ApprovalRecord, ApprovalStatus};
use ibkr_agent_gateway::testing::config::PaperTradingConfig;
use ibkr_agent_gateway::testing::domain::{
    AccountId, BrokerOrderId, ContractId, CurrencyCode, ErrorCode, LocalUserId, Money,
    OrderIntentId, OrderPreviewId, OrderSide, PreviewOrderType, Quantity, TimeInForce,
    ValidatedOrder, ValidatedOrderId,
};
use ibkr_agent_gateway::testing::orders::{
    IdempotencyKey, IdempotencyStore, PaperCancelRequest, PaperOrderLifecycleStatus,
    PaperSubmitRequest, cancel_paper_order, submit_paper_order,
};
use rust_decimal::Decimal;
use time::{Duration, OffsetDateTime};

#[test]
fn paper_submit_and_cancel_return_lifecycle_records() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let config = PaperTradingConfig {
        enabled: true,
        allowed_accounts: vec![account_id.clone()],
    };
    let mut idempotency_store = IdempotencyStore::default();
    let submit = submit_paper_order(
        PaperSubmitRequest {
            order: validated_order(account_id.clone())?,
            approval: approval(account_id.clone()),
            idempotency_key: IdempotencyKey::new("submit-key")?,
            paper_config: config.clone(),
        },
        &mut idempotency_store,
    )?;

    assert_eq!(
        submit.lifecycle.status,
        PaperOrderLifecycleStatus::Submitted
    );

    let cancel = cancel_paper_order(
        PaperCancelRequest {
            account_id,
            broker_order_id: BrokerOrderId::from_static("paper-order-local"),
            idempotency_key: IdempotencyKey::new("cancel-key")?,
            paper_config: config,
        },
        &mut idempotency_store,
    )?;

    assert_eq!(
        cancel.lifecycle.status,
        PaperOrderLifecycleStatus::Cancelled
    );
    Ok(())
}

#[test]
fn paper_submit_replays_same_request_and_rejects_conflicts()
-> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let config = PaperTradingConfig {
        enabled: true,
        allowed_accounts: vec![account_id.clone()],
    };
    let request = PaperSubmitRequest {
        order: validated_order(account_id.clone())?,
        approval: approval(account_id.clone()),
        idempotency_key: IdempotencyKey::new("submit-replay-key")?,
        paper_config: config.clone(),
    };
    let mut idempotency_store = IdempotencyStore::default();

    let first = submit_paper_order(request.clone(), &mut idempotency_store)?;
    let replayed = submit_paper_order(request.clone(), &mut idempotency_store)?;
    assert_eq!(first.idempotency_key, replayed.idempotency_key);
    assert_eq!(
        first.lifecycle.broker_order_id,
        replayed.lifecycle.broker_order_id
    );

    let conflict = PaperSubmitRequest {
        order: validated_order(account_id)?,
        approval: request.approval,
        idempotency_key: request.idempotency_key,
        paper_config: config,
    };
    let Err(error) = submit_paper_order(conflict, &mut idempotency_store) else {
        return Err("conflicting paper submit idempotency key should be rejected".into());
    };
    assert_eq!(error.code, ErrorCode::PaperIdempotencyConflict);
    Ok(())
}

fn account_id() -> Result<AccountId, Box<dyn std::error::Error>> {
    let Some(account_id) = AccountId::new("DU1234567") else {
        return Err("static account id rejected".into());
    };
    Ok(account_id)
}

fn approval(account_id: AccountId) -> ApprovalRecord {
    ApprovalRecord {
        approval_id: ApprovalId::new(),
        preview_id: OrderPreviewId::new(),
        account_id,
        approved_by: LocalUserId::from_static("local-user"),
        status: ApprovalStatus::Approved,
        approved_at: Some(OffsetDateTime::now_utc()),
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    }
}

fn validated_order(account_id: AccountId) -> Result<ValidatedOrder, Box<dyn std::error::Error>> {
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err("static currency rejected".into());
    };
    Ok(ValidatedOrder {
        validated_order_id: ValidatedOrderId::new(),
        intent_id: OrderIntentId::new(),
        account_id,
        contract_id: ContractId::from_static("265598"),
        side: OrderSide::Buy,
        quantity: Quantity::new(Decimal::ONE),
        order_type: PreviewOrderType::Limit,
        limit_price: Some(Money {
            amount: Decimal::new(100, 0),
            currency,
        }),
        time_in_force: TimeInForce::Day,
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
        warnings: Vec::new(),
    })
}
