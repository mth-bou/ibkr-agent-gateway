//! Paper submit flow.

use super::idempotency::{IdempotencyKey, IdempotencyStore, stable_request_hash};
use super::lifecycle::{PaperOrderLifecycleRecord, PaperOrderLifecycleStatus};
use super::paper_writer::PaperOrderWriter;
use crate::internal::approval::{ApprovalRecord, ApprovalStatus};
use crate::internal::config::PaperTradingConfig;
use crate::internal::domain::{ErrorCode, GatewayError, ValidatedOrder};
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
    /// Approval after successful one-time consumption.
    pub consumed_approval: ApprovalRecord,
}

/// Validates and submits a paper order through a [`PaperOrderWriter`].
pub async fn submit_paper_order(
    request: PaperSubmitRequest,
    writer: &dyn PaperOrderWriter,
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

    let now = OffsetDateTime::now_utc();
    super::approval_gate::validate_approved_preview(
        &request.approval,
        &request.order,
        now,
        ErrorCode::PaperApprovalRequired,
        "A matching approved preview is required",
        "Approve the preview before paper submit",
    )?;

    let request_hash = stable_request_hash(
        "paper.submit",
        &PaperSubmitFingerprint {
            order: &request.order,
            approval: &request.approval,
        },
    )?;
    let idempotency_key = request.idempotency_key.clone();
    idempotency_store.record_or_replay(idempotency_key.clone(), request_hash)?;

    let receipt = writer
        .submit_paper(&request.order, &idempotency_key)
        .await?;
    let mut consumed_approval = request.approval.clone();
    consumed_approval.status = ApprovalStatus::Consumed;

    Ok(PaperSubmitResult {
        lifecycle: PaperOrderLifecycleRecord {
            account_id: request.order.account_id,
            broker_order_id: receipt.broker_order_id,
            status: PaperOrderLifecycleStatus::Submitted,
            updated_at: now,
        },
        idempotency_key,
        consumed_approval,
    })
}

#[derive(Serialize)]
struct PaperSubmitFingerprint<'a> {
    order: &'a ValidatedOrder,
    approval: &'a ApprovalRecord,
}
