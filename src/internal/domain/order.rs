//! Read-only order records and forbidden write states.

use super::identifiers::{AccountId, BrokerOrderId, ContractId};
use super::money::{CurrencyCode, Money, Quantity};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Order side for existing broker records.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    /// Buy side.
    Buy,
    /// Sell side.
    Sell,
}

/// Read-only order status values.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReadOnlyOrderStatus {
    /// Order is open at the broker.
    Open,
    /// Order is filled.
    Filled,
    /// Order is cancelled.
    Cancelled,
    /// Order status is unknown but read-only.
    Unknown,
}

/// Existing broker order, status, or execution data viewed read-only.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReadOnlyOrderRecord {
    /// Account id.
    pub account_id: AccountId,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Optional broker/client correlation id, e.g. IBKR `cOID`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "cOID",
        alias = "clientOrderId"
    )]
    pub client_order_id: Option<String>,
    /// Order status.
    pub status: ReadOnlyOrderStatus,
    /// Order side.
    pub side: Option<OrderSide>,
    /// Original quantity.
    pub quantity: Option<Quantity>,
    /// Filled quantity.
    pub filled_quantity: Option<Quantity>,
    /// Contract id.
    pub contract_id: Option<ContractId>,
    /// Limit price.
    pub limit_price: Option<Money>,
    /// Currency.
    pub currency: Option<CurrencyCode>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    #[schemars(with = "Option<String>")]
    pub created_at: Option<OffsetDateTime>,
    /// Update timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    #[schemars(with = "Option<String>")]
    pub updated_at: Option<OffsetDateTime>,
}

/// Generic write actions that are refused outside explicit safer flows.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForbiddenWriteAction {
    /// Order preview.
    Preview,
    /// Order approval.
    Approve,
    /// Order submission.
    Submit,
    /// Order cancellation.
    Cancel,
    /// Order modification.
    Modify,
}
