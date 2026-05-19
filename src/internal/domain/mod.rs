//! Pure domain types for the IBKR agent gateway.
//!
//! This crate must stay free of HTTP, MCP, OAuth, storage, config loading, and
//! provider SDK concerns.

pub mod account;
pub mod contract;
pub mod error;
pub mod identifiers;
pub mod market;
pub mod money;
pub mod order;
pub mod order_history;
pub mod order_preview;
pub mod pnl;

pub use account::{
    AccountCapabilityProfile, AccountMarginProfile, AccountMode, BrokerAccount, BrokerBackendKind,
    BrokerSessionStatus, BrokerSessionVisibility,
};
pub use contract::{AssetClass, ContractCandidate};
pub use error::{ErrorCode, GatewayError};
pub use identifiers::{
    AccountId, AccountIdHash, AuditEventId, BrokerOrderId, ContractId, LocalUserId, RequestId,
    SessionId,
};
pub use market::{
    HistoricalBar, HistoricalBarsRequest, MarketDataPolicy, MarketDataStatus, MarketSnapshot,
    MissingTimestampPolicy, StalePolicy,
};
pub use money::{CurrencyCode, Money, Quantity};
pub use order::{ForbiddenWriteAction, OrderSide, ReadOnlyOrderRecord, ReadOnlyOrderStatus};
pub use order_history::{HistoricalOrderRecord, OrdersHistory, OrdersHistoryRequest};
pub use order_preview::{
    OrderContractInput, OrderIntent, OrderIntentId, OrderPreview, OrderPreviewId, PreviewOrderType,
    TimeInForce, ValidatedOrder, ValidatedOrderId,
};
pub use pnl::{PnlRealtime, PnlRealtimeRow, PnlSnapshot};

#[cfg(test)]
mod tests {
    use super::{AccountId, CurrencyCode, ErrorCode, GatewayError, MarketDataPolicy};

    #[test]
    fn rejects_empty_account_id() {
        assert!(AccountId::new("  ").is_none());
    }

    #[test]
    fn normalizes_currency_code() {
        let Some(currency) = CurrencyCode::new("usd") else {
            unreachable!("valid currency should be accepted");
        };
        assert_eq!(currency.as_str(), "USD");
    }

    #[test]
    fn rejects_invalid_currency_code() {
        assert!(CurrencyCode::new("US").is_none());
    }

    #[test]
    fn market_data_policy_default_is_positive() {
        let policy = MarketDataPolicy::default();
        assert!(policy.max_snapshot_age_seconds > 0);
    }

    #[test]
    fn gateway_error_carries_stable_code() {
        let error = GatewayError::new(
            ErrorCode::ReadonlyWriteForbidden,
            "write refused",
            false,
            None,
        );
        assert_eq!(error.code, ErrorCode::ReadonlyWriteForbidden);
    }
}
