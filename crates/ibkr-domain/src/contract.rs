//! Contract and instrument models.

use crate::identifiers::ContractId;
use crate::money::CurrencyCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// MVP-supported asset classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    /// Common stock.
    Stock,
    /// Exchange-traded fund.
    Etf,
}

/// Possible instrument returned by contract search.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ContractCandidate {
    /// Broker contract id.
    pub contract_id: ContractId,
    /// Instrument symbol.
    pub symbol: String,
    /// Safe broker description.
    pub description: Option<String>,
    /// Asset class.
    pub asset_class: AssetClass,
    /// Exchange or route.
    pub exchange: Option<String>,
    /// Trading currency.
    pub currency: CurrencyCode,
    /// Whether this candidate is a unique match.
    pub is_unique_match: bool,
}
