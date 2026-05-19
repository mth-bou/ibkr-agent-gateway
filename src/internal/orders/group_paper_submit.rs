//! Paper bracket submit flow.

use super::{
    IdempotencyKey, IdempotencyStore, PaperOrderGroupWriter, idempotency::stable_request_hash,
};
use crate::internal::config::PaperTradingConfig;
use crate::internal::domain::{
    ErrorCode, GatewayError, OrderGroupLifecycle, OrderGroupStatus, ValidatedOrderGroup,
};

/// Paper group submit request.
#[derive(Clone, Debug)]
pub struct PaperGroupSubmitRequest {
    /// Validated group.
    pub group: ValidatedOrderGroup,
    /// Idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// Paper config.
    pub paper_config: PaperTradingConfig,
}

/// Submits a paper group through the configured writer.
pub async fn submit_paper_group_order(
    request: PaperGroupSubmitRequest,
    writer: &dyn PaperOrderGroupWriter,
    idempotency_store: &mut IdempotencyStore,
) -> Result<OrderGroupLifecycle, GatewayError> {
    if !request.paper_config.enabled {
        return Err(GatewayError::new(
            ErrorCode::PaperTradingDisabled,
            "Paper group trading is disabled",
            false,
            Some("Enable paper trading explicitly".to_string()),
        ));
    }
    if !request
        .paper_config
        .allowed_accounts
        .contains(&request.group.account_id)
    {
        return Err(GatewayError::new(
            ErrorCode::PaperTradingDisabled,
            "Account is not in the paper trading allowlist",
            false,
            Some("Use an allowlisted paper account".to_string()),
        ));
    }
    let request_hash = stable_request_hash("paper.group.submit", &request.group)?;
    idempotency_store.record_or_replay(request.idempotency_key.clone(), request_hash)?;
    let receipt = writer
        .submit_paper_group(&request.group, &request.idempotency_key)
        .await?;
    Ok(OrderGroupLifecycle {
        group_id: request.group.group_id,
        account_id: request.group.account_id,
        broker_order_ids: receipt.broker_order_ids,
        status: OrderGroupStatus::Submitted,
    })
}
