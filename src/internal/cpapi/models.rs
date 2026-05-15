//! Client Portal Gateway response models used by the read-only adapter.

use serde::{Deserialize, Serialize};

/// Session/authentication status response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CpapiSessionResponse {
    /// Whether the broker session can serve requests.
    pub authenticated: bool,
    /// Whether manual action is required.
    pub competing: Option<bool>,
    /// Safe status message.
    pub message: Option<String>,
}

/// Keepalive/tickle response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CpapiTickleResponse {
    /// Whether the keepalive succeeded.
    pub ok: bool,
    /// Safe status message.
    pub message: Option<String>,
}

/// Accounts response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CpapiAccountsResponse {
    /// Account records.
    pub accounts: Vec<CpapiAccount>,
}

/// One safe account record from CPAPI mapping.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CpapiAccount {
    /// Raw broker account id.
    pub account_id: String,
    /// Optional display label.
    pub account_label: Option<String>,
    /// Account mode.
    pub account_mode: Option<String>,
    /// Base currency.
    pub base_currency: Option<String>,
}

/// Generic JSON wrapper for portfolio/account reads that are mapped later.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CpapiJsonResponse {
    /// Raw safe JSON payload.
    pub value: serde_json::Value,
}

/// Contract search response.
pub type CpapiContractsResponse = Vec<super::models::CpapiContractCandidate>;

/// Contract candidate response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CpapiContractCandidate {
    /// Contract id.
    pub contract_id: String,
    /// Symbol.
    pub symbol: String,
    /// Description.
    pub description: Option<String>,
    /// Asset class.
    pub asset_class: String,
    /// Exchange.
    pub exchange: Option<String>,
    /// Currency.
    pub currency: String,
    /// Unique match marker.
    pub is_unique_match: bool,
}

/// Market snapshot response.
pub type CpapiMarketSnapshotResponse = serde_json::Value;

/// Historical bars response.
pub type CpapiHistoricalBarsResponse = serde_json::Value;

/// Orders response.
pub type CpapiOrdersResponse = serde_json::Value;

/// Executions response.
pub type CpapiExecutionsResponse = serde_json::Value;
