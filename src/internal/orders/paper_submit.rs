//! Paper submit flow.

use super::idempotency::{IdempotencyKey, IdempotencyStore, stable_request_hash};
use super::lifecycle::{PaperOrderLifecycleRecord, PaperOrderLifecycleStatus};
use ibkr_approval::{ApprovalRecord, ApprovalStatus};
use ibkr_config::PaperTradingConfig;
use ibkr_domain::{BrokerOrderId, ErrorCode, GatewayError, ValidatedOrder};
use serde::Serialize;
use time::OffsetDateTime;

/// Paper submit request.
#[derive(Clone, Debug)]
pub struct PaperSubmitRequest {
    /// Validated preview-only order.
    pub order: ValidatedOrder,
    /// Approval record.
    pub approval: ApprovalRecord,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Paper config.
    pub paper_config: PaperTradingConfig,
}

/// Paper submit result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaperSubmitResult {
    /// Lifecycle record.
    pub lifecycle: PaperOrderLifecycleRecord,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
}

/// Validates and records a paper submit candidate without live trading.
pub fn submit_paper_order(
    request: PaperSubmitRequest,
    idempotency_store: &mut IdempotencyStore,
) -> Result<PaperSubmitResult, GatewayError> {
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
        .contains(&request.order.account_id)
    {
        return Err(GatewayError::new(
            ErrorCode::PaperTradingDisabled,
            "Account is not in the paper trading allowlist",
            false,
            Some("Use an allowlisted paper account".to_string()),
        ));
    }

    if request.approval.status != ApprovalStatus::Approved
        || request.approval.account_id != request.order.account_id
    {
        return Err(GatewayError::new(
            ErrorCode::PaperApprovalRequired,
            "A matching approved preview is required",
            false,
            Some("Approve the preview before paper submit".to_string()),
        ));
    }

    let request_hash = stable_request_hash(
        "paper.submit",
        &PaperSubmitFingerprint {
            order: &request.order,
            approval: &request.approval,
        },
    )?;
    let idempotency_key = request.idempotency_key.clone();
    idempotency_store.record_or_replay(idempotency_key.clone(), request_hash)?;

    Ok(PaperSubmitResult {
        lifecycle: PaperOrderLifecycleRecord {
            account_id: request.order.account_id,
            broker_order_id: BrokerOrderId::from_static("paper-order-local"),
            status: PaperOrderLifecycleStatus::Submitted,
            updated_at: OffsetDateTime::now_utc(),
        },
        idempotency_key,
    })
}

#[derive(Serialize)]
struct PaperSubmitFingerprint<'a> {
    order: &'a ValidatedOrder,
    approval: &'a ApprovalRecord,
}
