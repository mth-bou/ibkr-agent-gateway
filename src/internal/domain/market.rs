//! Market snapshot and historical bar models.

use super::identifiers::ContractId;
use super::money::{CurrencyCode, Money, Quantity};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Market data availability status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketDataStatus {
    /// Live market data.
    Live,
    /// Delayed market data.
    Delayed,
    /// Data is older than configured freshness.
    Stale,
    /// Data is unavailable.
    Unavailable,
}

/// Behavior for stale snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StalePolicy {
    /// Return data with warnings.
    Warn,
    /// Refuse stale data.
    Refuse,
}

/// Behavior for snapshots without timestamps.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MissingTimestampPolicy {
    /// Refuse missing timestamps.
    Refuse,
}

/// Local market-data freshness policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarketDataPolicy {
    /// Maximum snapshot age before stale handling.
    pub max_snapshot_age_seconds: u64,
    /// Whether delayed data can be returned with labels.
    pub allow_delayed: bool,
    /// Stale data behavior.
    pub stale_policy: StalePolicy,
    /// Missing timestamp behavior.
    pub missing_timestamp_policy: MissingTimestampPolicy,
}

impl Default for MarketDataPolicy {
    fn default() -> Self {
        Self {
            max_snapshot_age_seconds: 900,
            allow_delayed: true,
            stale_policy: StalePolicy::Warn,
            missing_timestamp_policy: MissingTimestampPolicy::Refuse,
        }
    }
}

/// Read-only market snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarketSnapshot {
    /// Contract id.
    pub contract_id: ContractId,
    /// Bid price.
    pub bid: Option<Money>,
    /// Ask price.
    pub ask: Option<Money>,
    /// Last traded price.
    pub last: Option<Money>,
    /// Snapshot currency.
    pub currency: CurrencyCode,
    /// Broker-provided source timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub source_timestamp: OffsetDateTime,
    /// Gateway receive timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub received_at: OffsetDateTime,
    /// Data status.
    pub data_status: MarketDataStatus,
    /// Snapshot staleness.
    pub staleness_seconds: u64,
    /// Safe warnings.
    pub warnings: Vec<String>,
}

/// Read-only historical bars request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalBarsRequest {
    /// Contract id.
    pub contract_id: ContractId,
    /// Broker duration string.
    pub duration: String,
    /// Broker bar size string.
    pub bar_size: String,
    /// Whether outside regular trading hours is requested.
    pub outside_regular_trading_hours: bool,
}

/// One read-only historical bar.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalBar {
    /// Bar timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub timestamp: OffsetDateTime,
    /// Open price.
    pub open: Money,
    /// High price.
    pub high: Money,
    /// Low price.
    pub low: Money,
    /// Close price.
    pub close: Money,
    /// Volume.
    pub volume: Quantity,
    /// Bar currency.
    pub currency: CurrencyCode,
}
