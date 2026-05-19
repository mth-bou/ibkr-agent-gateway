//! Grouped order domain models.

use super::{AccountId, BrokerOrderId, OrderPreview, ValidatedOrder};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bracket/group identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct OrderGroupId(Uuid);

impl OrderGroupId {
    /// Creates a fresh group id.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for OrderGroupId {
    fn default() -> Self {
        Self::new()
    }
}

/// Three-leg bracket preview.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BracketOrderPreview {
    /// Group id.
    pub group_id: OrderGroupId,
    /// Parent entry preview.
    pub parent: OrderPreview,
    /// Take-profit child preview.
    pub take_profit: OrderPreview,
    /// Stop-loss child preview.
    pub stop_loss: OrderPreview,
}

/// Server-owned validated bracket group.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ValidatedOrderGroup {
    /// Group id.
    pub group_id: OrderGroupId,
    /// Account id.
    pub account_id: AccountId,
    /// Parent entry order.
    pub parent: ValidatedOrder,
    /// Take-profit child order.
    pub take_profit: ValidatedOrder,
    /// Stop-loss child order.
    pub stop_loss: ValidatedOrder,
}

/// Submitted group lifecycle.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OrderGroupLifecycle {
    /// Group id.
    pub group_id: OrderGroupId,
    /// Account id.
    pub account_id: AccountId,
    /// Broker ids returned by the group writer.
    pub broker_order_ids: Vec<BrokerOrderId>,
    /// Group status.
    pub status: OrderGroupStatus,
}

/// Group lifecycle status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderGroupStatus {
    /// Group accepted by the configured writer.
    Submitted,
}
