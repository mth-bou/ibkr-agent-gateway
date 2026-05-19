//! Order intent validation entrypoint.

use super::checks::run_risk_checks;
use super::policy::{RiskDecision, RiskPolicy};
use crate::internal::domain::{ErrorCode, GatewayError, OrderIntent, PreviewOrderType};
use rust_decimal::Decimal;

/// Validates typed intent shape and then runs deterministic policy checks.
pub fn validate_order_intent(
    intent: &OrderIntent,
    policy: &RiskPolicy,
) -> Result<RiskDecision, GatewayError> {
    if matches!(intent.order_type, PreviewOrderType::Limit) && intent.limit_price.is_none() {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Limit order preview requires a limit price",
            false,
            Some("Provide a limit price".to_string()),
        ));
    }
    if matches!(intent.order_type, PreviewOrderType::Stop) && intent.stop_price.is_none() {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Stop order preview requires a stop price",
            false,
            Some("Provide a stop price".to_string()),
        ));
    }
    if matches!(intent.order_type, PreviewOrderType::StopLimit)
        && (intent.stop_price.is_none() || intent.limit_price.is_none())
    {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Stop-limit order preview requires stop and limit prices",
            false,
            Some("Provide stop and limit prices".to_string()),
        ));
    }
    if matches!(intent.order_type, PreviewOrderType::TrailingStop) {
        match (&intent.trailing_amount, intent.trailing_percent) {
            (Some(_), Some(_)) | (None, None) => {
                return Err(GatewayError::new(
                    ErrorCode::OrderValidationFailed,
                    "Trailing stop preview requires exactly one trailing amount or percent",
                    false,
                    Some("Provide one trailing offset".to_string()),
                ));
            }
            (Some(amount), None) if amount.amount <= Decimal::ZERO => {
                return Err(GatewayError::new(
                    ErrorCode::OrderValidationFailed,
                    "Trailing amount must be positive",
                    false,
                    Some("Use a positive trailing amount".to_string()),
                ));
            }
            (None, Some(percent)) if percent <= Decimal::ZERO => {
                return Err(GatewayError::new(
                    ErrorCode::OrderValidationFailed,
                    "Trailing percent must be positive",
                    false,
                    Some("Use a positive trailing percent".to_string()),
                ));
            }
            _ => {}
        }
    }

    Ok(run_risk_checks(intent, policy))
}
