//! Shared bounded order modification fields.

use crate::internal::domain::{ErrorCode, GatewayError, Money, Quantity, TimeInForce};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Bounded fields that may be changed on an existing order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrderModifyFields {
    /// New quantity.
    pub quantity: Option<Quantity>,
    /// New limit price.
    pub limit_price: Option<Money>,
    /// New stop price.
    pub stop_price: Option<Money>,
    /// New time in force.
    pub time_in_force: Option<TimeInForce>,
    /// New trailing amount.
    pub trailing_amount: Option<Money>,
    /// New trailing percent.
    pub trailing_percent: Option<Decimal>,
}

impl OrderModifyFields {
    /// Returns true when at least one bounded field is present.
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.quantity.is_some()
            || self.limit_price.is_some()
            || self.stop_price.is_some()
            || self.time_in_force.is_some()
            || self.trailing_amount.is_some()
            || self.trailing_percent.is_some()
    }

    /// Validates combinations that would otherwise map to ambiguous broker fields.
    pub fn validate(&self) -> Result<(), GatewayError> {
        if self.stop_price.is_some()
            && (self.trailing_amount.is_some() || self.trailing_percent.is_some())
        {
            return Err(GatewayError::new(
                ErrorCode::OrderValidationFailed,
                "Order modify cannot combine stop price and trailing stop fields",
                false,
                Some("Choose either stop_price or trailing_amount/trailing_percent".to_string()),
            ));
        }
        if self.trailing_amount.is_some() && self.trailing_percent.is_some() {
            return Err(GatewayError::new(
                ErrorCode::OrderValidationFailed,
                "Order modify cannot combine trailing amount and trailing percent",
                false,
                Some("Choose either trailing_amount or trailing_percent".to_string()),
            ));
        }
        Ok(())
    }
}
