//! Paper order commands.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use crate::internal::approval::ApprovalId;
use crate::internal::audit::SqliteAuditWriter;
use crate::internal::config::PaperTradingConfig;
use crate::internal::domain::{
    BrokerOrderId, ContractId, CurrencyCode, ErrorCode, GatewayError, Money, OrderIntentId,
    OrderSide, PreviewOrderType, Quantity, TimeInForce, ValidatedOrder, ValidatedOrderId,
};
use crate::internal::orders::{
    IdempotencyKey, IdempotencyStore, PaperCancelRequest, PaperSubmitRequest, cancel_paper_order,
    stable_request_hash, submit_paper_order,
};
use rust_decimal::Decimal;
use serde::Serialize;
use time::{Duration, OffsetDateTime};

const PAPER_SUBMIT_HUMAN_OUTPUT: &str = "paper order candidate recorded";
const PAPER_CANCEL_HUMAN_OUTPUT: &str = "paper cancel candidate recorded";

/// Runs a paper submit command.
pub async fn submit(
    audit_writer: &SqliteAuditWriter,
    account: &str,
    approval_id: &str,
    idempotency_key: &str,
    enable_paper: bool,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let approval_id = ApprovalId::parse(approval_id)?;
    let approval = audit_writer
        .load_approval(&approval_id)
        .await?
        .ok_or_else(missing_approval)?;
    let idempotency_key = IdempotencyKey::new(idempotency_key)?;
    let request_hash = stable_request_hash(
        "cli.paper.submit",
        &PaperSubmitCliFingerprint {
            account,
            approval_id: approval_id.as_uuid().to_string(),
        },
    )?;
    if let Some(payload) = audit_writer
        .replay_order_idempotency(&idempotency_key, &request_hash)
        .await?
    {
        return print_output(json, "paper order candidate replayed", &payload);
    }

    let request = PaperSubmitRequest {
        order: dummy_validated_order(account_id.clone())?,
        approval,
        idempotency_key: idempotency_key.clone(),
        paper_config: paper_config(account_id, enable_paper),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let result = submit_paper_order(request, &mut idempotency_store)?;
    let payload = serde_json::to_value(&result.lifecycle).map_err(|_| output_payload_error())?;
    audit_writer
        .insert_order_idempotency(&idempotency_key, &request_hash, &payload)
        .await?;
    print_output(json, PAPER_SUBMIT_HUMAN_OUTPUT, &result.lifecycle)
}

/// Runs a paper cancel command.
pub async fn cancel(
    audit_writer: &SqliteAuditWriter,
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
    let idempotency_key = IdempotencyKey::new(idempotency_key)?;
    let request_hash = stable_request_hash(
        "cli.paper.cancel",
        &PaperCancelCliFingerprint {
            account,
            broker_order_id: broker_order_id.as_str(),
        },
    )?;
    if let Some(payload) = audit_writer
        .replay_order_idempotency(&idempotency_key, &request_hash)
        .await?
    {
        return print_output(json, "paper cancel candidate replayed", &payload);
    }

    let request = PaperCancelRequest {
        account_id: account_id.clone(),
        broker_order_id,
        idempotency_key: idempotency_key.clone(),
        paper_config: paper_config(account_id, enable_paper),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let result = cancel_paper_order(request, &mut idempotency_store)?;
    let payload = serde_json::to_value(&result.lifecycle).map_err(|_| output_payload_error())?;
    audit_writer
        .insert_order_idempotency(&idempotency_key, &request_hash, &payload)
        .await?;
    print_output(json, PAPER_CANCEL_HUMAN_OUTPUT, &result.lifecycle)
}

fn paper_config(
    account_id: crate::internal::domain::AccountId,
    enabled: bool,
) -> PaperTradingConfig {
    PaperTradingConfig {
        enabled,
        allowed_accounts: vec![account_id],
    }
}

fn dummy_validated_order(
    account_id: crate::internal::domain::AccountId,
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

#[derive(Serialize)]
struct PaperSubmitCliFingerprint<'a> {
    account: &'a str,
    approval_id: String,
}

#[derive(Serialize)]
struct PaperCancelCliFingerprint<'a> {
    account: &'a str,
    broker_order_id: &'a str,
}

fn missing_approval() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Paper submit requires an existing approval record",
        false,
        Some("Run approvals create and pass its approval_id".to_string()),
    )
}

fn output_payload_error() -> GatewayError {
    GatewayError::new(
        ErrorCode::AuditWriteFailed,
        "Unable to serialize order lifecycle for idempotency",
        true,
        Some("Retry the order workflow".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::{PAPER_CANCEL_HUMAN_OUTPUT, PAPER_SUBMIT_HUMAN_OUTPUT};

    #[test]
    fn paper_human_outputs_describe_local_candidates_not_broker_execution() {
        assert_eq!(PAPER_SUBMIT_HUMAN_OUTPUT, "paper order candidate recorded");
        assert_eq!(PAPER_CANCEL_HUMAN_OUTPUT, "paper cancel candidate recorded");
        assert!(!PAPER_SUBMIT_HUMAN_OUTPUT.contains("submitted"));
        assert!(!PAPER_CANCEL_HUMAN_OUTPUT.contains("cancelled"));
    }
}
