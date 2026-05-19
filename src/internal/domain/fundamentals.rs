//! Fundamentals read models.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Bounded fundamentals response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FundamentalsReport {
    /// Symbol.
    pub symbol: String,
    /// Report date or broker timestamp.
    pub as_of: Option<String>,
    /// Provider-specific safe ratios/fields.
    pub fields: serde_json::Value,
}
