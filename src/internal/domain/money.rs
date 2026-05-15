//! Decimal-safe money and quantity types.

use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ISO-like currency code used for broker money values.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Creates an uppercase three-letter currency code.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into().trim().to_ascii_uppercase();
        if value.len() == 3
            && value
                .chars()
                .all(|character| character.is_ascii_alphabetic())
        {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the currency code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Decimal monetary value with an explicit currency.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Money {
    /// Amount represented as a decimal, never floating point.
    #[schemars(with = "String")]
    pub amount: Decimal,
    /// Currency for the amount.
    pub currency: CurrencyCode,
}

/// Decimal quantity value for positions and orders.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Quantity {
    /// Quantity represented as a decimal, never floating point.
    #[schemars(with = "String")]
    pub value: Decimal,
}

impl Quantity {
    /// Creates a quantity.
    #[must_use]
    pub const fn new(value: Decimal) -> Self {
        Self { value }
    }
}
