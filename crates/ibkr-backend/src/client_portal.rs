//! Client Portal Gateway backend implementation.

use crate::r#trait::{BackendResult, IbkrBackend};
use async_trait::async_trait;
use ibkr_cpapi::{ClientPortalClient, map_account, map_session_response, map_tickle_response};
use ibkr_domain::{
    AccountId, BrokerAccount, ContractCandidate, ContractId, HistoricalBar, HistoricalBarsRequest,
    MarketSnapshot, ReadOnlyOrderRecord,
};

/// Broker backend backed by a local Client Portal Gateway.
#[derive(Clone)]
pub struct ClientPortalBackend {
    client: ClientPortalClient,
}

impl ClientPortalBackend {
    /// Creates a Client Portal backend.
    #[must_use]
    pub const fn new(client: ClientPortalClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IbkrBackend for ClientPortalBackend {
    async fn session_status(&self) -> BackendResult<ibkr_domain::BrokerSessionStatus> {
        let response = self.client.session_status().await?;
        Ok(map_session_response(response))
    }

    async fn keepalive(&self) -> BackendResult<ibkr_domain::BrokerSessionStatus> {
        let response = self.client.tickle().await?;
        Ok(map_tickle_response(response))
    }

    async fn list_accounts(&self) -> BackendResult<Vec<BrokerAccount>> {
        let response = self.client.accounts().await?;
        response.accounts.into_iter().map(map_account).collect()
    }

    async fn account_summary(&self, _account_id: &AccountId) -> BackendResult<serde_json::Value> {
        Err(unimplemented_capability())
    }

    async fn positions(&self, _account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        Err(unimplemented_capability())
    }

    async fn search_contracts(&self, _query: &str) -> BackendResult<Vec<ContractCandidate>> {
        Err(unimplemented_capability())
    }

    async fn resolve_contract(&self, _query: &str) -> BackendResult<ContractCandidate> {
        Err(unimplemented_capability())
    }

    async fn market_snapshot(&self, _contract_id: &ContractId) -> BackendResult<MarketSnapshot> {
        Err(unimplemented_capability())
    }

    async fn historical_bars(
        &self,
        _request: &HistoricalBarsRequest,
    ) -> BackendResult<Vec<HistoricalBar>> {
        Err(unimplemented_capability())
    }

    async fn orders(&self, _account_id: &AccountId) -> BackendResult<Vec<ReadOnlyOrderRecord>> {
        Err(unimplemented_capability())
    }

    async fn order_status(
        &self,
        _account_id: &AccountId,
        _broker_order_id: &str,
    ) -> BackendResult<ReadOnlyOrderRecord> {
        Err(unimplemented_capability())
    }

    async fn executions(&self, _account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        Err(unimplemented_capability())
    }
}

fn unimplemented_capability() -> ibkr_domain::GatewayError {
    ibkr_domain::GatewayError::new(
        ibkr_domain::ErrorCode::BrokerCapabilityUnavailable,
        "Capability is implemented by a later read-only story",
        false,
        Some("Use a command from the current story".to_string()),
    )
}
