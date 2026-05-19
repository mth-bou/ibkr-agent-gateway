//! Shared bounded order modification fields.

use crate::internal::domain::{Money, Quantity, TimeInForce};
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
}
