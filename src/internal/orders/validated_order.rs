//! Validated order construction.

use crate::internal::domain::{
    ContractCandidate, GatewayError, OrderIntent, ValidatedOrder, ValidatedOrderId,
};
use time::{Duration, OffsetDateTime};

/// Builds a non-executable validated order from a typed intent and resolved contract.
pub fn build_validated_order(
    intent: &OrderIntent,
    contract: &ContractCandidate,
    warnings: Vec<String>,
    ttl_seconds: u64,
) -> Result<ValidatedOrder, GatewayError> {
    let ttl = i64::try_from(ttl_seconds).map_err(|_| {
        GatewayError::new(
            crate::internal::domain::ErrorCode::ConfigInvalid,
            "Preview TTL is too large",
            false,
            Some("Use a smaller preview expiration".to_string()),
        )
    })?;

    Ok(ValidatedOrder {
        validated_order_id: ValidatedOrderId::new(),
        preview_id: crate::internal::domain::OrderPreviewId::new(),
        intent_id: intent.intent_id.clone(),
        account_id: intent.account_id.clone(),
        contract_id: contract.contract_id.clone(),
        symbol: Some(contract.symbol.clone()),
        asset_class: Some(contract.asset_class),
        side: intent.side,
        quantity: intent.quantity.clone(),
        order_type: intent.order_type,
        limit_price: intent.limit_price.clone(),
        stop_price: intent.stop_price.clone(),
        trailing_amount: intent.trailing_amount.clone(),
        trailing_percent: intent.trailing_percent,
        time_in_force: intent.time_in_force,
        expires_at: OffsetDateTime::now_utc() + Duration::seconds(ttl),
        warnings,
    })
}
