//! Paper order commands.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use crate::internal::approval::ApprovalId;
use crate::internal::audit::{
    OrderIdempotencyOperation, OrderIdempotencyRecoveryContext, OrderIdempotencyWorkflow,
    SqliteAuditWriter,
};
use crate::internal::config::PaperTradingConfig;
use crate::internal::domain::{BrokerOrderId, ErrorCode, GatewayError};
use crate::internal::orders::{
    IdempotencyKey, IdempotencyStore, LocalCandidatePaperWriter, PaperCancelRequest,
    PaperSubmitRequest, cancel_paper_order, handle_pending_order_error, stable_request_hash,
    submit_paper_order,
};
use serde::Serialize;

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
    let preview_record = audit_writer
        .load_order_preview(&approval.preview_id)
        .await?
        .ok_or_else(missing_preview)?;
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
        order: preview_record.validated_order,
        approval,
        idempotency_key: idempotency_key.clone(),
        paper_config: paper_config(account_id, enable_paper),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidatePaperWriter;
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Paper,
        operation: OrderIdempotencyOperation::Submit,
        account_id: request.order.account_id.clone(),
        broker_order_id: None,
    };
    audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;
    let result = match submit_paper_order(request, &writer, &mut idempotency_store).await {
        Ok(result) => result,
        Err(error) => {
            handle_pending_order_error(audit_writer, &idempotency_key, &request_hash, &error)
                .await?;
            return Err(error);
        }
    };
    let payload = serde_json::to_value(&result.lifecycle).map_err(|_| output_payload_error())?;
    audit_writer
        .complete_order_workflow(
            &idempotency_key,
            &request_hash,
            &payload,
            std::slice::from_ref(&result.consumed_approval),
        )
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
    let writer = LocalCandidatePaperWriter;
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Paper,
        operation: OrderIdempotencyOperation::Cancel,
        account_id: request.account_id.clone(),
        broker_order_id: Some(request.broker_order_id.clone()),
    };
    audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;
    let result = match cancel_paper_order(request, &writer, &mut idempotency_store).await {
        Ok(result) => result,
        Err(error) => {
            handle_pending_order_error(audit_writer, &idempotency_key, &request_hash, &error)
                .await?;
            return Err(error);
        }
    };
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

fn missing_preview() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Paper submit requires the approved preview to be present",
        false,
        Some("Create a fresh preview and approval before paper submit".to_string()),
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
