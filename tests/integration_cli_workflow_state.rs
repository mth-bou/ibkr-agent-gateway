#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::approval::ApprovalService;
use ibkr_agent_gateway::testing::audit::{AuditHmacKey, SqliteAuditWriter};
use ibkr_agent_gateway::testing::domain::{
    AccountId, AssetClass, AuditEventId, ContractId, CurrencyCode, ErrorCode, LocalUserId, Money,
    OrderIntentId, OrderPreviewId, OrderSide, PreviewOrderType, Quantity, TimeInForce,
    ValidatedOrder, ValidatedOrderId,
};
use ibkr_agent_gateway::testing::orders::create_order_preview;
use rust_decimal::Decimal;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

#[tokio::test]
async fn missing_config_path_is_not_silently_replaced_by_fake_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    let result = ibkr_agent_gateway::cli::run_from_args([
        "ibkr-agent",
        "--config",
        "/tmp/ibkr-agent-gateway-missing-config.yaml",
        "accounts",
        "list",
        "--json",
    ])
    .await;

    let Err(error) = result else {
        return Err("missing config path must fail".into());
    };
    assert_eq!(error.code, ErrorCode::ConfigInvalid);
    Ok(())
}

#[tokio::test]
async fn paper_submit_requires_persisted_approval_and_replays_idempotency()
-> Result<(), Box<dyn std::error::Error>> {
    let key = Arc::new(AuditHmacKey::ephemeral()?);
    let writer = SqliteAuditWriter::connect("sqlite::memory:", key).await?;
    let account = AccountId::from_static("DU1234567");
    let mut approval_service = ApprovalService::default();
    let order = validated_order(account.clone())?;
    let preview = create_order_preview(&order, AuditEventId::new(), None, None)?;
    writer.append_order_preview(&preview, &order).await?;

    let approval = approval_service.create_approval(
        order.preview_id.clone(),
        account.clone(),
        LocalUserId::from_static("local-user"),
        300,
    );
    writer.append_approval(&approval).await?;
    let approval_id = approval.approval_id.as_uuid().to_string();

    ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        &approval_id,
        "integration-paper-idempotency-key",
        true,
        true,
    )
    .await?;

    ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        &approval_id,
        "integration-paper-idempotency-key",
        true,
        true,
    )
    .await?;

    let consumed = ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        &approval_id,
        "integration-paper-consumed-approval-key",
        true,
        true,
    )
    .await;
    let Err(consumed) = consumed else {
        return Err("consumed approval must refuse a fresh submit".into());
    };
    assert_eq!(consumed.code, ErrorCode::ApprovalConsumed);

    let conflict_order = validated_order(account.clone())?;
    let conflict_preview = create_order_preview(&conflict_order, AuditEventId::new(), None, None)?;
    writer
        .append_order_preview(&conflict_preview, &conflict_order)
        .await?;
    let conflicting_approval = approval_service.create_approval(
        conflict_order.preview_id.clone(),
        account.clone(),
        LocalUserId::from_static("local-user"),
        300,
    );
    writer.append_approval(&conflicting_approval).await?;
    let conflict = ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        &conflicting_approval.approval_id.as_uuid().to_string(),
        "integration-paper-idempotency-key",
        true,
        true,
    )
    .await;
    let Err(conflict) = conflict else {
        return Err("same idempotency key with different request must be refused".into());
    };
    assert_eq!(conflict.code, ErrorCode::PaperIdempotencyConflict);

    let missing = ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        "019e35d0-aa66-7d33-a057-680d56300b57",
        "integration-paper-missing-approval-key",
        true,
        true,
    )
    .await;
    let Err(missing) = missing else {
        return Err("unknown approval id must be refused".into());
    };
    assert_eq!(missing.code, ErrorCode::PaperApprovalRequired);
    Ok(())
}

#[tokio::test]
async fn approval_create_requires_persisted_preview() -> Result<(), Box<dyn std::error::Error>> {
    let key = Arc::new(AuditHmacKey::ephemeral()?);
    let writer = SqliteAuditWriter::connect("sqlite::memory:", key).await?;
    let preview_id = OrderPreviewId::new().as_uuid().to_string();

    let result = ibkr_agent_gateway::cli::commands::approvals::create(
        &writer,
        "DU1234567",
        &preview_id,
        300,
        true,
    )
    .await;
    let Err(error) = result else {
        return Err("approval create must reject unknown preview ids".into());
    };
    assert_eq!(error.code, ErrorCode::PaperApprovalRequired);
    Ok(())
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
