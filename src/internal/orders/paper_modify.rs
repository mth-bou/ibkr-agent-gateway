//! Paper modify flow.

use super::{
    IdempotencyKey, IdempotencyStore, OrderModifyFields, PaperOrderWriter,
    idempotency::stable_request_hash,
    lifecycle::{PaperOrderLifecycleRecord, PaperOrderLifecycleStatus},
};
use crate::internal::config::PaperTradingConfig;
use crate::internal::domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError};
use serde::Serialize;
use time::OffsetDateTime;

/// Paper modify request.
#[derive(Clone, Debug)]
pub struct PaperModifyRequest {
    /// Account id.
    pub account_id: AccountId,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Bounded changes.
    pub changes: OrderModifyFields,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Paper config.
    pub paper_config: PaperTradingConfig,
}

/// Paper modify result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaperModifyResult {
    /// Lifecycle record.
    pub lifecycle: PaperOrderLifecycleRecord,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
}

/// Validates and modifies a paper order through a [`PaperOrderWriter`].
pub async fn modify_paper_order(
    request: PaperModifyRequest,
    writer: &dyn PaperOrderWriter,
    idempotency_store: &mut IdempotencyStore,
) -> Result<PaperModifyResult, GatewayError> {
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
    if !request.changes.has_changes() {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Paper modify requires at least one bounded change",
            false,
            Some("Provide quantity, price, time-in-force, or trailing changes".to_string()),
        ));
    }

    let request_hash = stable_request_hash(
        "paper.modify",
        &ModifyFingerprint {
            account_id: &request.account_id,
            broker_order_id: &request.broker_order_id,
            changes: &request.changes,
        },
    )?;
    let idempotency_key = request.idempotency_key.clone();
    idempotency_store.record_or_replay(idempotency_key.clone(), request_hash)?;
    let receipt = writer
        .modify_paper(
            &request.account_id,
            &request.broker_order_id,
            &request.changes,
            &idempotency_key,
        )
        .await?;

    Ok(PaperModifyResult {
        lifecycle: PaperOrderLifecycleRecord {
            account_id: request.account_id,
            broker_order_id: receipt.broker_order_id,
            status: PaperOrderLifecycleStatus::Open,
            updated_at: OffsetDateTime::now_utc(),
        },
        idempotency_key,
    })
}

#[derive(Serialize)]
struct ModifyFingerprint<'a> {
    account_id: &'a AccountId,
    broker_order_id: &'a BrokerOrderId,
    changes: &'a OrderModifyFields,
}
