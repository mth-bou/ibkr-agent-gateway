//! Paper order lifecycle models.

use ibkr_domain::{AccountId, BrokerOrderId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Paper order lifecycle states.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperOrderLifecycleStatus {
    /// Submit was accepted locally or by paper broker.
    Submitted,
    /// Paper order is open.
    Open,
    /// Paper order filled.
    Filled,
    /// Paper order cancelled.
    Cancelled,
    /// Paper order refused.
    Refused,
}

/// Paper order lifecycle record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaperOrderLifecycleRecord {
    /// Account id.
    pub account_id: AccountId,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Current status.
    pub status: PaperOrderLifecycleStatus,
    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
