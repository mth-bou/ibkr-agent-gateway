//! Order lifecycle models.

use crate::internal::domain::{AccountId, BrokerOrderId, Money, ReadOnlyOrderStatus};
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

/// Live order lifecycle states.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveOrderLifecycleStatus {
    /// Live submit was accepted by the local gateway path.
    Submitted,
    /// Live order is open.
    Open,
    /// Live cancel was accepted and is awaiting broker completion.
    PendingCancel,
    /// Live order was fully filled.
    Filled,
    /// Live order was cancelled.
    Cancelled,
    /// Live order was refused.
    Refused,
}

impl LiveOrderLifecycleStatus {
    /// Returns true when the broker lifecycle can be removed from polling.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Filled | Self::Cancelled | Self::Refused)
    }

    /// Converts a read-only broker status into the live lifecycle model.
    #[must_use]
    pub const fn from_read_only_order_status(status: ReadOnlyOrderStatus) -> Self {
        match status {
            ReadOnlyOrderStatus::Open => Self::Open,
            ReadOnlyOrderStatus::Filled => Self::Filled,
            ReadOnlyOrderStatus::Cancelled => Self::Cancelled,
            ReadOnlyOrderStatus::Unknown => Self::Submitted,
        }
    }

    /// Converts a broker cancel response into the live lifecycle model.
    #[must_use]
    pub fn from_cancel_receipt_status(broker_status: Option<&str>, accepted: bool) -> Self {
        match broker_status.map(normalize_broker_status).as_deref() {
            Some("cancelled" | "canceled") => Self::Cancelled,
            Some("pendingcancel") => Self::PendingCancel,
            Some("filled") => Self::Filled,
            Some("rejected" | "refused" | "inactive") => Self::Refused,
            Some("submitted" | "presubmitted" | "open") => Self::Open,
            Some(_) | None if accepted => Self::PendingCancel,
            Some(_) | None => Self::Refused,
        }
    }
}

fn normalize_broker_status(status: &str) -> String {
    status
        .chars()
        .filter(|character| !matches!(character, '_' | '-' | ' '))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Live execution correlation without raw broker payloads.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LiveExecutionCorrelation {
    /// Safe execution correlation id.
    pub correlation_id: String,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Last correlation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// Live order lifecycle record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LiveOrderLifecycleRecord {
    /// Account id.
    pub account_id: AccountId,
    /// Broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Current status.
    pub status: LiveOrderLifecycleStatus,
    /// Estimated submitted notional for live session-limit accounting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notional: Option<Money>,
    /// Optional execution correlation.
    pub execution_correlation: Option<LiveExecutionCorrelation>,
    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
