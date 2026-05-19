//! Options chain and greek read models.

use super::{ContractId, CurrencyCode, Money};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One option contract in a chain.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OptionChainEntry {
    /// Broker contract id for the option.
    pub contract_id: ContractId,
    /// Underlying symbol.
    pub underlying_symbol: String,
    /// Expiration date as broker-provided `YYYY-MM-DD`.
    pub expiration: String,
    /// Strike price.
    pub strike: Money,
    /// `call` or `put`.
    pub right: OptionRight,
    /// Exchange or route.
    pub exchange: Option<String>,
}

/// Option right.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OptionRight {
    /// Call option.
    Call,
    /// Put option.
    Put,
}

/// Option chain result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OptionChain {
    /// Underlying symbol.
    pub underlying_symbol: String,
    /// Currency.
    pub currency: CurrencyCode,
    /// Chain entries.
    pub entries: Vec<OptionChainEntry>,
}

/// Option greek snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OptionGreeks {
    /// Option contract id.
    pub contract_id: ContractId,
    /// Delta.
    #[schemars(with = "Option<String>")]
    pub delta: Option<Decimal>,
    /// Gamma.
    #[schemars(with = "Option<String>")]
    pub gamma: Option<Decimal>,
    /// Theta.
    #[schemars(with = "Option<String>")]
    pub theta: Option<Decimal>,
    /// Vega.
    #[schemars(with = "Option<String>")]
    pub vega: Option<Decimal>,
    /// Implied volatility.
    #[schemars(with = "Option<String>")]
    pub implied_volatility: Option<Decimal>,
}
