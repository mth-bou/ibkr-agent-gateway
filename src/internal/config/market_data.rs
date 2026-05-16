//! Market-data policy validation.

use crate::internal::domain::{ErrorCode, GatewayError, MarketDataPolicy};

/// Validates local market-data freshness policy.
pub fn validate_market_data_policy(policy: &MarketDataPolicy) -> Result<(), GatewayError> {
    if policy.max_snapshot_age_seconds == 0 {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            "market_data.max_snapshot_age_seconds must be positive",
            false,
            Some("Set max_snapshot_age_seconds to a positive value".to_string()),
        ));
    }

    Ok(())
}
