//! Non-executable order preview construction.

use ibkr_domain::{
    AuditEventId, ErrorCode, GatewayError, Money, OrderPreview, OrderPreviewId, ValidatedOrder,
};
use rust_decimal::Decimal;

/// Creates a preview result without calling broker submit/cancel endpoints.
pub fn create_order_preview(
    order: &ValidatedOrder,
    audit_event_id: AuditEventId,
    estimated_commission: Option<Money>,
    margin_impact: Option<Money>,
) -> Result<OrderPreview, GatewayError> {
    let Some(limit_price) = &order.limit_price else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Preview requires a priced validated order",
            false,
            Some("Validate a limit order before preview".to_string()),
        ));
    };

    let estimated_cost = Money {
        amount: limit_price.amount * order.quantity.value.max(Decimal::ZERO),
        currency: limit_price.currency.clone(),
    };

    Ok(OrderPreview {
        preview_id: OrderPreviewId::new(),
        validated_order_id: order.validated_order_id.clone(),
        estimated_cost,
        estimated_commission,
        margin_impact,
        warnings: order.warnings.clone(),
        expires_at: order.expires_at,
        audit_event_id,
    })
}
