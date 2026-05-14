//! Validated order construction.

use ibkr_domain::{ContractId, GatewayError, OrderIntent, ValidatedOrder, ValidatedOrderId};
use time::{Duration, OffsetDateTime};

/// Builds a non-executable validated order from a typed intent and resolved contract.
pub fn build_validated_order(
    intent: &OrderIntent,
    contract_id: ContractId,
    warnings: Vec<String>,
    ttl_seconds: u64,
) -> Result<ValidatedOrder, GatewayError> {
    let ttl = i64::try_from(ttl_seconds).map_err(|_| {
        GatewayError::new(
            ibkr_domain::ErrorCode::ConfigInvalid,
            "Preview TTL is too large",
            false,
            Some("Use a smaller preview expiration".to_string()),
        )
    })?;

    Ok(ValidatedOrder {
        validated_order_id: ValidatedOrderId::new(),
        intent_id: intent.intent_id.clone(),
        account_id: intent.account_id.clone(),
        contract_id,
        side: intent.side,
        quantity: intent.quantity.clone(),
        order_type: intent.order_type,
        limit_price: intent.limit_price.clone(),
        time_in_force: intent.time_in_force,
        expires_at: OffsetDateTime::now_utc() + Duration::seconds(ttl),
        warnings,
    })
}
