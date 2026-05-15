use ibkr_approval::{ApprovalId, ApprovalRecord, ApprovalService, ApprovalStatus};
use ibkr_config::PaperTradingConfig;
use ibkr_domain::{
    AccountId, ContractId, CurrencyCode, ErrorCode, LocalUserId, Money, OrderIntentId,
    OrderPreviewId, OrderSide, PreviewOrderType, Quantity, TimeInForce, ValidatedOrder,
    ValidatedOrderId,
};
use ibkr_orders::{IdempotencyKey, IdempotencyStore, PaperSubmitRequest, submit_paper_order};
use rust_decimal::Decimal;
use time::{Duration, OffsetDateTime};

#[test]
fn approval_service_creates_readable_approval() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let mut service = ApprovalService::default();
    let approval = service.create_approval(
        OrderPreviewId::new(),
        account_id,
        LocalUserId::from_static("local-user"),
        300,
    );

    let stored = service.get(&approval.approval_id);
    assert!(stored.is_some());
    Ok(())
}

#[test]
fn paper_submit_requires_approved_record() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let request = PaperSubmitRequest {
        order: validated_order(account_id.clone())?,
        approval: ApprovalRecord {
            approval_id: ApprovalId::new(),
            preview_id: OrderPreviewId::new(),
            account_id: account_id.clone(),
            approved_by: LocalUserId::from_static("local-user"),
            status: ApprovalStatus::Pending,
            approved_at: None,
            expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
        },
        idempotency_key: IdempotencyKey::new("paper-submit-1")?,
        paper_config: PaperTradingConfig {
            enabled: true,
            allowed_accounts: vec![account_id],
        },
    };

    let mut idempotency_store = IdempotencyStore::default();
    let Err(error) = submit_paper_order(request, &mut idempotency_store) else {
        return Err("pending approval unexpectedly allowed submit".into());
    };
    assert_eq!(error.code, ErrorCode::PaperApprovalRequired);
    Ok(())
}

fn account_id() -> Result<AccountId, Box<dyn std::error::Error>> {
    let Some(account_id) = AccountId::new("DU1234567") else {
        return Err("static account id rejected".into());
    };
    Ok(account_id)
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
