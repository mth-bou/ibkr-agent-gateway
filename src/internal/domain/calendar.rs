//! Market calendar read models.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Market session status.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarketSession {
    /// Exchange or market code.
    pub exchange: String,
    /// Whether the market is currently open.
    pub is_open: bool,
    /// Next open timestamp as broker-provided RFC3339 text.
    pub next_open: Option<String>,
    /// Next close timestamp as broker-provided RFC3339 text.
    pub next_close: Option<String>,
}

/// Market holiday row.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarketHoliday {
    /// Exchange or market code.
    pub exchange: String,
    /// Holiday date.
    pub date: String,
    /// Holiday name.
    pub name: String,
}

/// Market holidays response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarketHolidays {
    /// Exchange or market code.
    pub exchange: String,
    /// Holiday rows.
    pub holidays: Vec<MarketHoliday>,
}
