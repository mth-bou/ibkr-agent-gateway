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
