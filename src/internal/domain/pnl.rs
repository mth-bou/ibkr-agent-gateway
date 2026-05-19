//! Read-only PnL models.

use super::identifiers::AccountId;
use super::market::MarketDataStatus;
use super::money::Money;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Daily account PnL snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PnlSnapshot {
    /// Account id.
    pub account_id: AccountId,
    /// Realized PnL for the requested period.
    pub realized_pnl: Money,
    /// Unrealized PnL for the requested period.
    pub unrealized_pnl: Money,
    /// Total PnL for the requested period.
    pub total_pnl: Money,
    /// Period start.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub period_start: OffsetDateTime,
    /// Period end.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub period_end: OffsetDateTime,
    /// Gateway or broker timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub timestamp: OffsetDateTime,
    /// Data availability status.
    pub data_status: MarketDataStatus,
}

/// One row in a realtime PnL response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PnlRealtimeRow {
    /// Optional contract id or broker row key.
    pub key: String,
    /// Optional display symbol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Current row PnL.
    pub pnl: Money,
}

/// Realtime account PnL snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PnlRealtime {
    /// Account id.
    pub account_id: AccountId,
    /// Bounded broker rows.
    pub rows: Vec<PnlRealtimeRow>,
    /// Total current PnL.
    pub total_pnl: Money,
    /// Gateway or broker timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub timestamp: OffsetDateTime,
    /// Data availability status.
    pub data_status: MarketDataStatus,
}
