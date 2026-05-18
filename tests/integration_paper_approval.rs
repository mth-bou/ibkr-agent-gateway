use ibkr_agent_gateway::testing::approval::{
    ApprovalId, ApprovalRecord, ApprovalService, ApprovalStatus,
};
use ibkr_agent_gateway::testing::config::PaperTradingConfig;
use ibkr_agent_gateway::testing::domain::{
    AccountId, AssetClass, ContractId, CurrencyCode, ErrorCode, LocalUserId, Money, OrderIntentId,
    OrderPreviewId, OrderSide, PreviewOrderType, Quantity, TimeInForce, ValidatedOrder,
    ValidatedOrderId,
};
use ibkr_agent_gateway::testing::orders::{
    IdempotencyKey, IdempotencyStore, LocalCandidatePaperWriter, PaperSubmitRequest,
    submit_paper_order,
};
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

#[tokio::test]
async fn paper_submit_requires_approved_record() -> Result<(), Box<dyn std::error::Error>> {
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
    let writer = LocalCandidatePaperWriter;
    let Err(error) = submit_paper_order(request, &writer, &mut idempotency_store).await else {
        return Err("pending approval unexpectedly allowed submit".into());
    };
    assert_eq!(error.code, ErrorCode::PaperApprovalRequired);
    Ok(())
}

#[tokio::test]
async fn paper_submit_refuses_mismatched_preview_id() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let order = validated_order(account_id.clone())?;
    let request = PaperSubmitRequest {
        order,
        approval: ApprovalRecord {
            approval_id: ApprovalId::new(),
            preview_id: OrderPreviewId::new(),
            account_id: account_id.clone(),
            approved_by: LocalUserId::from_static("local-user"),
            status: ApprovalStatus::Approved,
            approved_at: Some(OffsetDateTime::now_utc()),
            expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
        },
        idempotency_key: IdempotencyKey::new("paper-submit-preview-mismatch")?,
        paper_config: PaperTradingConfig {
            enabled: true,
            allowed_accounts: vec![account_id],
        },
    };

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidatePaperWriter;
    let Err(error) = submit_paper_order(request, &writer, &mut idempotency_store).await else {
        return Err("mismatched approval preview id unexpectedly allowed submit".into());
    };
    assert_eq!(error.code, ErrorCode::ApprovalPreviewMismatch);
    Ok(())
}

#[tokio::test]
async fn paper_submit_refuses_consumed_approval() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let order = validated_order(account_id.clone())?;
    let mut approval = approval_for_order(account_id.clone(), &order);
    approval.status = ApprovalStatus::Consumed;
    let request = PaperSubmitRequest {
        order,
        approval,
        idempotency_key: IdempotencyKey::new("paper-submit-consumed")?,
        paper_config: PaperTradingConfig {
            enabled: true,
            allowed_accounts: vec![account_id],
        },
    };

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidatePaperWriter;
    let Err(error) = submit_paper_order(request, &writer, &mut idempotency_store).await else {
        return Err("consumed approval unexpectedly allowed submit".into());
    };
    assert_eq!(error.code, ErrorCode::ApprovalConsumed);
    Ok(())
}

#[tokio::test]
async fn paper_submit_refuses_expired_approval() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let order = validated_order(account_id.clone())?;
    let mut approval = approval_for_order(account_id.clone(), &order);
    approval.expires_at = OffsetDateTime::now_utc() - Duration::seconds(1);
    let request = PaperSubmitRequest {
        order,
        approval,
        idempotency_key: IdempotencyKey::new("paper-submit-expired")?,
        paper_config: PaperTradingConfig {
            enabled: true,
            allowed_accounts: vec![account_id],
        },
    };

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidatePaperWriter;
    let Err(error) = submit_paper_order(request, &writer, &mut idempotency_store).await else {
        return Err("expired approval unexpectedly allowed submit".into());
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
        preview_id: OrderPreviewId::new(),
        intent_id: OrderIntentId::new(),
        account_id,
        contract_id: ContractId::from_static("265598"),
        symbol: Some("AAPL".to_string()),
        asset_class: Some(AssetClass::Stock),
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

fn approval_for_order(account_id: AccountId, order: &ValidatedOrder) -> ApprovalRecord {
    ApprovalRecord {
        approval_id: ApprovalId::new(),
        preview_id: order.preview_id.clone(),
        account_id,
        approved_by: LocalUserId::from_static("local-user"),
        status: ApprovalStatus::Approved,
        approved_at: Some(OffsetDateTime::now_utc()),
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    }
}
