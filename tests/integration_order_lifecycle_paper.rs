use async_trait::async_trait;
use ibkr_agent_gateway::testing::approval::{ApprovalId, ApprovalRecord, ApprovalStatus};
use ibkr_agent_gateway::testing::config::PaperTradingConfig;
use ibkr_agent_gateway::testing::domain::{
    AccountId, AssetClass, BrokerOrderId, ContractId, CurrencyCode, ErrorCode, LocalUserId, Money,
    OrderIntentId, OrderPreviewId, OrderSide, PreviewOrderType, Quantity, TimeInForce,
    ValidatedOrder, ValidatedOrderId,
};
use ibkr_agent_gateway::testing::orders::{
    IdempotencyKey, IdempotencyStore, LocalCandidatePaperWriter, OrderModifyFields,
    PaperCancelReceipt, PaperCancelRequest, PaperModifyReceipt, PaperModifyRequest,
    PaperOrderLifecycleStatus, PaperOrderWriter, PaperSubmitReceipt, PaperSubmitRequest,
    cancel_paper_order, modify_paper_order, submit_paper_order,
};
use rust_decimal::Decimal;
use time::{Duration, OffsetDateTime};

#[tokio::test]
async fn paper_submit_and_cancel_return_lifecycle_records() -> Result<(), Box<dyn std::error::Error>>
{
    let account_id = account_id()?;
    let config = PaperTradingConfig {
        enabled: true,
        allowed_accounts: vec![account_id.clone()],
    };
    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidatePaperWriter;
    let order = validated_order(account_id.clone())?;
    let submit = submit_paper_order(
        PaperSubmitRequest {
            approval: approval_for_order(account_id.clone(), &order),
            order,
            idempotency_key: IdempotencyKey::new("submit-key")?,
            paper_config: config.clone(),
        },
        &writer,
        &mut idempotency_store,
    )
    .await?;

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
        &writer,
        &mut idempotency_store,
    )
    .await?;

    assert_eq!(
        cancel.lifecycle.status,
        PaperOrderLifecycleStatus::Cancelled
    );
    Ok(())
}

#[tokio::test]
async fn paper_submit_replays_same_request_and_rejects_conflicts()
-> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let config = PaperTradingConfig {
        enabled: true,
        allowed_accounts: vec![account_id.clone()],
    };
    let order = validated_order(account_id.clone())?;
    let request = PaperSubmitRequest {
        approval: approval_for_order(account_id.clone(), &order),
        order,
        idempotency_key: IdempotencyKey::new("submit-replay-key")?,
        paper_config: config.clone(),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidatePaperWriter;

    let first = submit_paper_order(request.clone(), &writer, &mut idempotency_store).await?;
    let replayed = submit_paper_order(request.clone(), &writer, &mut idempotency_store).await?;
    assert_eq!(first.idempotency_key, replayed.idempotency_key);
    assert_eq!(
        first.lifecycle.broker_order_id,
        replayed.lifecycle.broker_order_id
    );

    let conflict_order = validated_order(account_id.clone())?;
    let conflict = PaperSubmitRequest {
        approval: approval_for_order(account_id, &conflict_order),
        order: conflict_order,
        idempotency_key: request.idempotency_key,
        paper_config: config,
    };
    let Err(error) = submit_paper_order(conflict, &writer, &mut idempotency_store).await else {
        return Err("conflicting paper submit idempotency key should be rejected".into());
    };
    assert_eq!(error.code, ErrorCode::PaperIdempotencyConflict);
    Ok(())
}

#[tokio::test]
async fn paper_submit_maps_negative_broker_status() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let order = validated_order(account_id.clone())?;
    let mut idempotency_store = IdempotencyStore::default();
    let submit = submit_paper_order(
        PaperSubmitRequest {
            approval: approval_for_order(account_id.clone(), &order),
            order,
            idempotency_key: IdempotencyKey::new("submit-rejected-key")?,
            paper_config: paper_config(account_id),
        },
        &StatusPaperWriter {
            submit_status: Some("Rejected".to_string()),
            cancel_accepted: true,
            cancel_status: Some("Cancelled".to_string()),
            modify_accepted: true,
            modify_status: Some("Modified".to_string()),
        },
        &mut idempotency_store,
    )
    .await?;

    assert_eq!(submit.lifecycle.status, PaperOrderLifecycleStatus::Refused);
    Ok(())
}

#[tokio::test]
async fn paper_cancel_rejects_unaccepted_active_status() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let mut idempotency_store = IdempotencyStore::default();
    let error = cancel_paper_order(
        PaperCancelRequest {
            account_id: account_id.clone(),
            broker_order_id: BrokerOrderId::from_static("paper-active"),
            idempotency_key: IdempotencyKey::new("cancel-active-key")?,
            paper_config: paper_config(account_id),
        },
        &StatusPaperWriter {
            submit_status: Some("Submitted".to_string()),
            cancel_accepted: false,
            cancel_status: Some("Submitted".to_string()),
            modify_accepted: true,
            modify_status: Some("Modified".to_string()),
        },
        &mut idempotency_store,
    )
    .await
    .err()
    .ok_or("unaccepted active cancel status should fail")?;

    assert_eq!(error.code, ErrorCode::BrokerResponseInvalid);
    Ok(())
}

#[tokio::test]
async fn paper_modify_rejects_unaccepted_active_status() -> Result<(), Box<dyn std::error::Error>> {
    let account_id = account_id()?;
    let currency = CurrencyCode::new("USD").ok_or("static currency rejected")?;
    let mut idempotency_store = IdempotencyStore::default();
    let error = modify_paper_order(
        PaperModifyRequest {
            account_id: account_id.clone(),
            broker_order_id: BrokerOrderId::from_static("paper-active"),
            changes: OrderModifyFields {
                quantity: None,
                limit_price: Some(Money {
                    amount: Decimal::new(101, 0),
                    currency,
                }),
                stop_price: None,
                time_in_force: None,
                trailing_amount: None,
                trailing_percent: None,
            },
            idempotency_key: IdempotencyKey::new("modify-active-key")?,
            paper_config: paper_config(account_id),
        },
        &StatusPaperWriter {
            submit_status: Some("Submitted".to_string()),
            cancel_accepted: true,
            cancel_status: Some("Cancelled".to_string()),
            modify_accepted: false,
            modify_status: Some("Submitted".to_string()),
        },
        &mut idempotency_store,
    )
    .await
    .err()
    .ok_or("unaccepted active modify status should fail")?;

    assert_eq!(error.code, ErrorCode::BrokerResponseInvalid);
    Ok(())
}

fn account_id() -> Result<AccountId, Box<dyn std::error::Error>> {
    let Some(account_id) = AccountId::new("DU1234567") else {
        return Err("static account id rejected".into());
    };
    Ok(account_id)
}

fn paper_config(account_id: AccountId) -> PaperTradingConfig {
    PaperTradingConfig {
        enabled: true,
        allowed_accounts: vec![account_id],
    }
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

fn approval_for_order(account_id: AccountId, order: &ValidatedOrder) -> ApprovalRecord {
    let mut approval = approval(account_id);
    approval.preview_id = order.preview_id.clone();
    approval
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
        stop_price: None,
        trailing_amount: None,
        trailing_percent: None,
        time_in_force: TimeInForce::Day,
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
        warnings: Vec::new(),
    })
}

struct StatusPaperWriter {
    submit_status: Option<String>,
    cancel_accepted: bool,
    cancel_status: Option<String>,
    modify_accepted: bool,
    modify_status: Option<String>,
}

#[async_trait]
impl PaperOrderWriter for StatusPaperWriter {
    async fn submit_paper(
        &self,
        _order: &ValidatedOrder,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<PaperSubmitReceipt, ibkr_agent_gateway::testing::domain::GatewayError> {
        Ok(PaperSubmitReceipt {
            broker_order_id: BrokerOrderId::from_static("paper-status"),
            broker_status: self.submit_status.clone(),
        })
    }

    async fn cancel_paper(
        &self,
        _account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<PaperCancelReceipt, ibkr_agent_gateway::testing::domain::GatewayError> {
        Ok(PaperCancelReceipt {
            broker_order_id: broker_order_id.clone(),
            accepted: self.cancel_accepted,
            broker_status: self.cancel_status.clone(),
        })
    }

    async fn modify_paper(
        &self,
        _account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        _changes: &OrderModifyFields,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<PaperModifyReceipt, ibkr_agent_gateway::testing::domain::GatewayError> {
        Ok(PaperModifyReceipt {
            broker_order_id: broker_order_id.clone(),
            accepted: self.modify_accepted,
            broker_status: self.modify_status.clone(),
        })
    }
}
