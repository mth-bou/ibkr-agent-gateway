//! Paper cancel flow.

use crate::idempotency::IdempotencyKey;
use crate::lifecycle::{PaperOrderLifecycleRecord, PaperOrderLifecycleStatus};
use ibkr_config::PaperTradingConfig;
use ibkr_domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError};
use time::OffsetDateTime;

/// Paper cancel request.
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

/// Validates and records a paper cancel candidate without live trading.
pub fn cancel_paper_order(request: PaperCancelRequest) -> Result<PaperCancelResult, GatewayError> {
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

    Ok(PaperCancelResult {
        lifecycle: PaperOrderLifecycleRecord {
            account_id: request.account_id,
            broker_order_id: request.broker_order_id,
            status: PaperOrderLifecycleStatus::Cancelled,
            updated_at: OffsetDateTime::now_utc(),
        },
        idempotency_key: request.idempotency_key,
    })
}
