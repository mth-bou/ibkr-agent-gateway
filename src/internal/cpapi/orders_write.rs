//! Client Portal paper write adapter boundary.
//!
//! This module is paper-only and contains no live trading adapter.

use crate::internal::domain::BrokerOrderId;
use serde::{Deserialize, Serialize};

/// Paper submit response mapped from a broker adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPortalPaperSubmitResponse {
    /// Paper broker order id.
    pub broker_order_id: BrokerOrderId,
}

/// Paper cancel response mapped from a broker adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPortalPaperCancelResponse {
    /// Paper broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Whether cancellation was accepted.
    pub accepted: bool,
}
