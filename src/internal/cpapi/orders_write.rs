//! Client Portal paper write adapter boundary.
//!
//! This module declares paper-only response types. The live trading adapter
//! lives in [`crate::internal::cpapi::live_writer`] and implements
//! [`crate::internal::orders::LiveOrderWriter`] against the Client Portal
//! Gateway order endpoints.

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
