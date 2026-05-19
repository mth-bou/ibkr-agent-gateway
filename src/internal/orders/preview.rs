//! Non-executable order preview construction.

use crate::internal::domain::{
    AuditEventId, ErrorCode, GatewayError, Money, OrderPreview, PreviewOrderType, ValidatedOrder,
};
use rust_decimal::Decimal;

/// Creates a preview result without calling broker submit/cancel endpoints.
pub fn create_order_preview(
    order: &ValidatedOrder,
    audit_event_id: AuditEventId,
    estimated_commission: Option<Money>,
    margin_impact: Option<Money>,
) -> Result<OrderPreview, GatewayError> {
    let price_basis = preview_price_basis(order)?;

    let estimated_cost = Money {
        amount: price_basis.amount * order.quantity.value.max(Decimal::ZERO),
        currency: price_basis.currency.clone(),
    };

    Ok(OrderPreview {
        preview_id: order.preview_id.clone(),
        validated_order_id: order.validated_order_id.clone(),
        estimated_cost,
        estimated_commission,
        margin_impact,
        warnings: order.warnings.clone(),
        expires_at: order.expires_at,
        audit_event_id,
    })
}

fn preview_price_basis(order: &ValidatedOrder) -> Result<&Money, GatewayError> {
    match order.order_type {
        PreviewOrderType::Limit => order.limit_price.as_ref(),
        PreviewOrderType::Stop => order.stop_price.as_ref(),
        PreviewOrderType::StopLimit => order.limit_price.as_ref().or(order.stop_price.as_ref()),
        PreviewOrderType::TrailingStop => order.trailing_amount.as_ref(),
        PreviewOrderType::Market => None,
    }
    .ok_or_else(|| {
        GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Preview requires a priced validated order",
            false,
            Some("Provide a limit, stop, or trailing amount for preview estimation".to_string()),
        )
    })
}
