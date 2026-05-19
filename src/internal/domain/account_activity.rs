//! Account activity read models.

use super::{AccountId, CurrencyCode, Money};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Currency rate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CurrencyRate {
    /// Base currency.
    pub base: CurrencyCode,
    /// Quote currency.
    pub quote: CurrencyCode,
    /// Rate.
    #[schemars(with = "String")]
    pub rate: Decimal,
}

/// Transfer activity row.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransferRecord {
    /// Transfer id.
    pub transfer_id: String,
    /// Transfer type.
    pub kind: String,
    /// Amount.
    pub amount: Money,
    /// Broker-provided date.
    pub date: String,
}

/// Transfer history.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransferHistory {
    /// Account id.
    pub account_id: AccountId,
    /// Transfer rows.
    pub transfers: Vec<TransferRecord>,
}
