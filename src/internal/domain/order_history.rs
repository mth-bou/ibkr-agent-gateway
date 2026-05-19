//! Read-only historical order models.

use super::identifiers::{AccountId, BrokerOrderId, ContractId};
use super::money::{Money, Quantity};
use super::order::{OrderSide, ReadOnlyOrderStatus};
use super::order_preview::{PreviewOrderType, TimeInForce};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Bounded order history request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OrdersHistoryRequest {
    /// Account id.
    pub account_id: AccountId,
    /// Optional start timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    #[schemars(with = "Option<String>")]
    pub from: Option<OffsetDateTime>,
    /// Optional end timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    #[schemars(with = "Option<String>")]
    pub to: Option<OffsetDateTime>,
    /// Optional status filter.
    pub status: Option<ReadOnlyOrderStatus>,
    /// Maximum records.
    pub limit: u32,
}

/// Historical broker order record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalOrderRecord {
    /// Account id.
    pub account_id: AccountId,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Contract id.
    pub contract_id: Option<ContractId>,
    /// Optional symbol.
    pub symbol: Option<String>,
    /// Order side.
    pub side: Option<OrderSide>,
    /// Quantity.
    pub quantity: Option<Quantity>,
    /// Filled quantity.
    pub filled_quantity: Option<Quantity>,
    /// Order type.
    pub order_type: Option<PreviewOrderType>,
    /// Limit price.
    pub limit_price: Option<Money>,
    /// Stop price.
    pub stop_price: Option<Money>,
    /// Time in force.
    pub time_in_force: Option<TimeInForce>,
    /// Status.
    pub status: ReadOnlyOrderStatus,
    /// Average fill price.
    pub average_fill_price: Option<Money>,
    /// Safe cancellation reason.
    pub cancel_reason: Option<String>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    #[schemars(with = "Option<String>")]
    pub created_at: Option<OffsetDateTime>,
    /// Update timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    #[schemars(with = "Option<String>")]
    pub updated_at: Option<OffsetDateTime>,
}

/// Bounded order history response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OrdersHistory {
    /// Account id.
    pub account_id: AccountId,
    /// Returned records.
    pub orders: Vec<HistoricalOrderRecord>,
    /// Whether more broker rows were omitted.
    pub truncated: bool,
}
