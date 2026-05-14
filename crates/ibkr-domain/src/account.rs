//! Broker account and session status models.

use crate::error::ErrorCode;
use crate::identifiers::{AccountId, AccountIdHash};
use crate::money::CurrencyCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Broker backend kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BrokerBackendKind {
    /// Local Interactive Brokers Client Portal Gateway.
    ClientPortalGateway,
    /// Deterministic fake backend for offline validation.
    Fake,
}

/// Externally visible broker session status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BrokerSessionVisibility {
    /// Broker session can serve read-only data.
    Usable,
    /// Manual user action is required.
    ManualActionRequired,
    /// Backend is unavailable.
    Unavailable,
}

/// Broker session status without secrets.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BrokerSessionStatus {
    /// Visible session status.
    pub status: BrokerSessionVisibility,
    /// Backend that produced the status.
    pub backend: BrokerBackendKind,
    /// Timestamp when the status was checked.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub checked_at: OffsetDateTime,
    /// Last successful keepalive timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    #[schemars(with = "Option<String>")]
    pub last_keepalive_at: Option<OffsetDateTime>,
    /// Safe manual action when required.
    pub user_action: Option<String>,
    /// Optional stable error code.
    pub error_code: Option<ErrorCode>,
}

/// Account mode reported by broker metadata or local config.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountMode {
    /// Paper account.
    Paper,
    /// Live account.
    Live,
    /// Mode is not known yet.
    Unknown,
}

/// Safe account metadata visible to the local user and MCP clients.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BrokerAccount {
    /// Raw broker account id, never used directly in audit.
    pub account_id: AccountId,
    /// HMAC account hash for audit correlation.
    pub account_id_hash: AccountIdHash,
    /// Optional safe display label.
    pub account_label: Option<String>,
    /// Paper/live/unknown mode.
    pub account_mode: AccountMode,
    /// Base currency when known.
    pub base_currency: Option<CurrencyCode>,
    /// Whether source metadata was redacted.
    pub metadata_redacted: bool,
}
