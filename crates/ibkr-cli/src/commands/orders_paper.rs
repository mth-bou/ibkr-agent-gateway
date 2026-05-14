//! Paper order commands.

use crate::{commands::account::parse_account_id, output::print_output};
use ibkr_approval::{ApprovalId, ApprovalRecord, ApprovalStatus};
use ibkr_config::PaperTradingConfig;
use ibkr_domain::{
    BrokerOrderId, ContractId, CurrencyCode, ErrorCode, GatewayError, LocalUserId, Money,
    OrderIntentId, OrderSide, PreviewOrderType, Quantity, TimeInForce, ValidatedOrder,
    ValidatedOrderId,
};
use ibkr_orders::{
    IdempotencyKey, PaperCancelRequest, PaperSubmitRequest, cancel_paper_order, submit_paper_order,
};
use rust_decimal::Decimal;
use time::{Duration, OffsetDateTime};

/// Runs a paper submit command.
pub fn submit(
    account: &str,
    idempotency_key: &str,
    enable_paper: bool,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let request = PaperSubmitRequest {
        order: dummy_validated_order(account_id.clone())?,
        approval: dummy_approval(account_id.clone()),
        idempotency_key: IdempotencyKey::new(idempotency_key)?,
        paper_config: paper_config(account_id, enable_paper),
    };
    let result = submit_paper_order(request)?;
    print_output(json, "paper order submitted", &result.lifecycle)
}

/// Runs a paper cancel command.
pub fn cancel(
    account: &str,
    broker_order_id: &str,
    idempotency_key: &str,
    enable_paper: bool,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let Some(broker_order_id) = BrokerOrderId::new(broker_order_id) else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Broker order id is required",
            false,
            Some("Provide a broker order id".to_string()),
        ));
    };
    let request = PaperCancelRequest {
        account_id: account_id.clone(),
        broker_order_id,
        idempotency_key: IdempotencyKey::new(idempotency_key)?,
        paper_config: paper_config(account_id, enable_paper),
    };
    let result = cancel_paper_order(request)?;
    print_output(json, "paper order cancelled", &result.lifecycle)
}

fn paper_config(account_id: ibkr_domain::AccountId, enabled: bool) -> PaperTradingConfig {
    PaperTradingConfig {
        enabled,
        allowed_accounts: vec![account_id],
    }
}

fn dummy_approval(account_id: ibkr_domain::AccountId) -> ApprovalRecord {
    ApprovalRecord {
        approval_id: ApprovalId::new(),
        preview_id: ibkr_domain::OrderPreviewId::new(),
        account_id,
        approved_by: LocalUserId::from_static("local-user"),
        status: ApprovalStatus::Approved,
        approved_at: Some(OffsetDateTime::now_utc()),
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    }
}

fn dummy_validated_order(
    account_id: ibkr_domain::AccountId,
) -> Result<ValidatedOrder, GatewayError> {
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Static currency is invalid",
            false,
            None,
        ));
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
