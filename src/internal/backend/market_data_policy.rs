//! Market-data stale/delayed policy application.

use crate::internal::domain::{
    ErrorCode, GatewayError, MarketDataPolicy, MarketDataStatus, StalePolicy,
};

/// Validates market-data status against local policy.
pub fn apply_market_data_policy(
    policy: &MarketDataPolicy,
    status: MarketDataStatus,
) -> Result<(), GatewayError> {
    match status {
        MarketDataStatus::Live => Ok(()),
        MarketDataStatus::Delayed if policy.allow_delayed => Ok(()),
        MarketDataStatus::Delayed => Err(GatewayError::new(
            ErrorCode::MarketDataDelayed,
            "Delayed market data is disabled",
            false,
            Some("Enable delayed data or use live market data".to_string()),
        )),
        MarketDataStatus::Stale if matches!(policy.stale_policy, StalePolicy::Warn) => Ok(()),
        MarketDataStatus::Stale => Err(GatewayError::new(
            ErrorCode::MarketDataStale,
            "Market data is stale",
            false,
            Some("Refresh market data or adjust stale policy".to_string()),
        )),
        MarketDataStatus::Unavailable => Err(GatewayError::new(
            ErrorCode::MarketDataUnavailable,
            "Market data is unavailable",
            true,
            Some("Check market data permissions or retry".to_string()),
        )),
    }
}
