//! Paper submit flow.

use crate::idempotency::IdempotencyKey;
use crate::lifecycle::{PaperOrderLifecycleRecord, PaperOrderLifecycleStatus};
use ibkr_approval::{ApprovalRecord, ApprovalStatus};
use ibkr_config::PaperTradingConfig;
use ibkr_domain::{BrokerOrderId, ErrorCode, GatewayError, ValidatedOrder};
use time::OffsetDateTime;

/// Paper submit request.
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
pub fn submit_paper_order(request: PaperSubmitRequest) -> Result<PaperSubmitResult, GatewayError> {
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

    Ok(PaperSubmitResult {
        lifecycle: PaperOrderLifecycleRecord {
            account_id: request.order.account_id,
            broker_order_id: BrokerOrderId::from_static("paper-order-local"),
            status: PaperOrderLifecycleStatus::Submitted,
            updated_at: OffsetDateTime::now_utc(),
        },
        idempotency_key: request.idempotency_key,
    })
}
