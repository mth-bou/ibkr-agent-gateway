//! Client Portal preview estimate mapping.
//!
//! This module intentionally contains no submit, cancel, or approval endpoint.

use ibkr_domain::Money;
use serde::{Deserialize, Serialize};

/// Optional broker-side preview estimate if a backend supports it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientPortalPreviewEstimate {
    /// Estimated cost.
    pub estimated_cost: Money,
    /// Estimated commission.
    pub estimated_commission: Option<Money>,
    /// Estimated margin impact.
    pub margin_impact: Option<Money>,
}
