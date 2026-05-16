//! Read-only broker backend trait.

use async_trait::async_trait;
use ibkr_domain::{
    AccountId, BrokerAccount, BrokerSessionStatus, ContractCandidate, ContractId, GatewayError,
    HistoricalBar, HistoricalBarsRequest, MarketSnapshot, ReadOnlyOrderRecord,
};

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

    /// Returns account summary as a typed JSON-compatible value until the
    /// portfolio model is introduced in US2.
    async fn account_summary(&self, account_id: &AccountId) -> BackendResult<serde_json::Value>;

    /// Returns a portfolio snapshot as a JSON-compatible value.
    async fn portfolio_snapshot(&self, account_id: &AccountId) -> BackendResult<serde_json::Value>;

    /// Returns positions as typed JSON-compatible values until the position
    /// model is introduced in US2.
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
        broker_order_id: &str,
    ) -> BackendResult<ReadOnlyOrderRecord>;

    /// Lists read-only executions as JSON-compatible values until execution
    /// models are introduced in US2.
    async fn executions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>>;
}
