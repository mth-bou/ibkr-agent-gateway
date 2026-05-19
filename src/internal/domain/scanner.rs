//! Market scanner read models.

use super::{ContractId, Money};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One scanner result row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScannerResult {
    /// Rank in the scanner result.
    pub rank: u32,
    /// Contract id.
    pub contract_id: ContractId,
    /// Symbol.
    pub symbol: String,
    /// Description.
    pub description: Option<String>,
    /// Last price if available.
    pub last: Option<Money>,
    /// Percent change if available.
    #[schemars(with = "Option<String>")]
    pub change_percent: Option<Decimal>,
}

/// Scanner response.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScannerRun {
    /// Scanner code.
    pub scanner_code: String,
    /// Result rows.
    pub results: Vec<ScannerResult>,
}
