//! Paper cancel flow.

use super::idempotency::{IdempotencyKey, IdempotencyStore, stable_request_hash};
use super::lifecycle::{PaperOrderLifecycleRecord, PaperOrderLifecycleStatus};
use super::paper_writer::PaperOrderWriter;
use crate::internal::config::PaperTradingConfig;
use crate::internal::domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError};
use serde::Serialize;
use time::OffsetDateTime;

/// Paper cancel request.
#[derive(Clone, Debug)]
pub struct PaperCancelRequest {
    /// Account id.
    pub account_id: AccountId,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Paper config.
    pub paper_config: PaperTradingConfig,
}

/// Paper cancel result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaperCancelResult {
    /// Lifecycle record.
    pub lifecycle: PaperOrderLifecycleRecord,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
}

/// Validates and cancels a paper order through a [`PaperOrderWriter`].
pub async fn cancel_paper_order(
    request: PaperCancelRequest,
    writer: &dyn PaperOrderWriter,
    idempotency_store: &mut IdempotencyStore,
) -> Result<PaperCancelResult, GatewayError> {
    if !request.paper_config.enabled {
        return Err(GatewayError::new(
            ErrorCode::PaperTradingDisabled,
            "Paper trading is disabled",
            false,
            Some("Enable paper trading explicitly".to_string()),
        ));
    }

    if !request
        .paper_config
        .allowed_accounts
        .contains(&request.account_id)
    {
        return Err(GatewayError::new(
            ErrorCode::PaperTradingDisabled,
            "Account is not in the paper trading allowlist",
            false,
            Some("Use an allowlisted paper account".to_string()),
        ));
    }

    let request_hash = stable_request_hash(
        "paper.cancel",
        &PaperCancelFingerprint {
            account_id: &request.account_id,
            broker_order_id: &request.broker_order_id,
        },
    )?;
    let idempotency_key = request.idempotency_key.clone();
    idempotency_store.record_or_replay(idempotency_key.clone(), request_hash)?;
    let receipt = writer
        .cancel_paper(
            &request.account_id,
            &request.broker_order_id,
            &idempotency_key,
        )
        .await?;

    let status = PaperOrderLifecycleStatus::from_cancel_receipt_status(
        receipt.broker_status.as_deref(),
        receipt.accepted,
    );
    if !receipt.accepted && !status.is_terminal() {
        return Err(GatewayError::new(
            ErrorCode::BrokerResponseInvalid,
            "Broker did not accept paper cancel",
            false,
            Some("Inspect broker order status before retrying cancel".to_string()),
        ));
    }

    Ok(PaperCancelResult {
        lifecycle: PaperOrderLifecycleRecord {
            account_id: request.account_id,
            broker_order_id: receipt.broker_order_id,
            status,
            updated_at: OffsetDateTime::now_utc(),
        },
        idempotency_key,
    })
}

#[derive(Serialize)]
struct PaperCancelFingerprint<'a> {
    account_id: &'a AccountId,
    broker_order_id: &'a BrokerOrderId,
}
