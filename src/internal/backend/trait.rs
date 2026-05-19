//! Read-only broker backend trait.

use crate::internal::domain::{
    AccountCapabilityProfile, AccountId, BrokerAccount, BrokerSessionStatus, ContractCandidate,
    ContractId, ErrorCode, GatewayError, HistoricalBar, HistoricalBarsRequest, MarketSnapshot,
    OrdersHistory, OrdersHistoryRequest, PnlRealtime, PnlSnapshot, ReadOnlyOrderRecord,
};
use crate::internal::domain::{MarketDepth, OptionChain, OptionGreeks, ScannerRun};
use async_trait::async_trait;

/// Result type for backend operations.
pub type BackendResult<T> = Result<T, GatewayError>;

/// Typed read-only broker backend boundary.
#[async_trait]
pub trait IbkrBackend: Send + Sync {
    /// Returns broker session status without secrets.
    async fn session_status(&self) -> BackendResult<BrokerSessionStatus>;

    /// Attempts a keepalive and returns the resulting session status.
    async fn keepalive(&self) -> BackendResult<BrokerSessionStatus>;

    /// Lists accounts visible to the current session.
    async fn list_accounts(&self) -> BackendResult<Vec<BrokerAccount>>;

    /// Returns account summary as a JSON-compatible broker payload.
    async fn account_summary(&self, account_id: &AccountId) -> BackendResult<serde_json::Value>;

    /// Returns a portfolio snapshot as a JSON-compatible value.
    async fn portfolio_snapshot(&self, account_id: &AccountId) -> BackendResult<serde_json::Value>;

    /// Returns positions as JSON-compatible broker payloads.
    async fn positions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>>;

    /// Searches contract candidates.
    async fn search_contracts(&self, query: &str) -> BackendResult<Vec<ContractCandidate>>;

    /// Resolves one contract from explicit context.
    async fn resolve_contract(&self, query: &str) -> BackendResult<ContractCandidate>;

    /// Returns a market snapshot.
    async fn market_snapshot(&self, contract_id: &ContractId) -> BackendResult<MarketSnapshot>;

    /// Returns historical bars.
    async fn historical_bars(
        &self,
        request: &HistoricalBarsRequest,
    ) -> BackendResult<Vec<HistoricalBar>>;

    /// Lists read-only orders.
    async fn orders(&self, account_id: &AccountId) -> BackendResult<Vec<ReadOnlyOrderRecord>>;

    /// Returns one read-only order status.
    async fn order_status(
        &self,
        account_id: &AccountId,
        order_lookup_id: &str,
    ) -> BackendResult<ReadOnlyOrderRecord>;

    /// Lists read-only executions as JSON-compatible broker payloads.
    async fn executions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>>;

    /// Returns daily account PnL.
    async fn pnl_daily(&self, account_id: &AccountId) -> BackendResult<PnlSnapshot>;

    /// Returns realtime account PnL.
    async fn pnl_realtime(&self, account_id: &AccountId) -> BackendResult<PnlRealtime>;

    /// Returns bounded historical orders.
    async fn orders_history(&self, request: &OrdersHistoryRequest) -> BackendResult<OrdersHistory>;

    /// Returns safe account capabilities and restrictions.
    async fn account_metadata(
        &self,
        account_id: &AccountId,
    ) -> BackendResult<AccountCapabilityProfile>;

    /// Returns a bounded options chain for an underlying symbol.
    async fn options_chain(&self, _symbol: &str) -> BackendResult<OptionChain> {
        Err(advanced_market_unavailable("options chain"))
    }

    /// Returns option greeks for one option contract.
    async fn option_greeks(&self, _contract_id: &ContractId) -> BackendResult<OptionGreeks> {
        Err(advanced_market_unavailable("option greeks"))
    }

    /// Returns a bounded level-II snapshot.
    async fn market_depth(&self, _contract_id: &ContractId) -> BackendResult<MarketDepth> {
        Err(advanced_market_unavailable("market depth"))
    }

    /// Runs a broker scanner.
    async fn scanner_run(&self, _scanner_code: &str) -> BackendResult<ScannerRun> {
        Err(advanced_market_unavailable("scanner"))
    }
}

fn advanced_market_unavailable(capability: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerCapabilityUnavailable,
        format!("Broker backend does not support {capability}"),
        false,
        Some("Use a backend with the requested market research capability".to_string()),
    )
}
