//! Order intent validation entrypoint.

use super::checks::run_risk_checks;
use super::policy::{RiskDecision, RiskPolicy};
use crate::internal::domain::{ErrorCode, GatewayError, OrderIntent, PreviewOrderType};

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

    Ok(run_risk_checks(intent, policy))
}
