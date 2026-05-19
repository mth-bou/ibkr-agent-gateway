//! Market depth read models.

use super::{ContractId, Money, Quantity};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One level-II order book row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarketDepthLevel {
    /// Price level.
    pub price: Money,
    /// Size at the level.
    pub size: Quantity,
    /// Venue or market maker.
    pub venue: Option<String>,
}

/// Bounded market depth snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarketDepth {
    /// Contract id.
    pub contract_id: ContractId,
    /// Bid-side levels.
    pub bids: Vec<MarketDepthLevel>,
    /// Ask-side levels.
    pub asks: Vec<MarketDepthLevel>,
}
