//! Client Portal Gateway backend implementation.

use super::r#trait::{BackendResult, IbkrBackend};
use async_trait::async_trait;
use ibkr_cpapi::{
    ClientPortalClient, map_account, map_contract_candidate, map_session_response,
    map_tickle_response,
};
use ibkr_domain::{
    AccountId, BrokerAccount, ContractCandidate, ContractId, ErrorCode, GatewayError,
    HistoricalBar, HistoricalBarsRequest, MarketSnapshot, ReadOnlyOrderRecord,
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

    async fn account_summary(&self, account_id: &AccountId) -> BackendResult<serde_json::Value> {
        Ok(self
            .client
            .account_summary(account_id.as_str())
            .await?
            .value)
    }

    async fn portfolio_snapshot(&self, account_id: &AccountId) -> BackendResult<serde_json::Value> {
        Ok(self
            .client
            .portfolio_snapshot(account_id.as_str())
            .await?
            .value)
    }

    async fn positions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        let value = self.client.positions(account_id.as_str()).await?.value;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn search_contracts(&self, query: &str) -> BackendResult<Vec<ContractCandidate>> {
        self.client
            .contracts_search(query)
            .await?
            .into_iter()
            .map(map_contract_candidate)
            .collect()
    }

    async fn resolve_contract(&self, query: &str) -> BackendResult<ContractCandidate> {
        let candidates = self.search_contracts(query).await?;
        let unique = candidates
            .iter()
            .filter(|candidate| candidate.is_unique_match)
            .cloned()
            .collect::<Vec<_>>();
        match unique.as_slice() {
            [candidate] => Ok(candidate.clone()),
            _ => Err(GatewayError::new(
                ErrorCode::InputAmbiguousContract,
                "Contract resolution is ambiguous",
                false,
                Some("Provide symbol, asset class, currency, and exchange".to_string()),
            )),
        }
    }

    async fn market_snapshot(&self, contract_id: &ContractId) -> BackendResult<MarketSnapshot> {
        let value = self.client.market_snapshot(contract_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn historical_bars(
        &self,
        request: &HistoricalBarsRequest,
    ) -> BackendResult<Vec<HistoricalBar>> {
        let value = self
            .client
            .historical_bars(
                request.contract_id.as_str(),
                &request.duration,
                &request.bar_size,
            )
            .await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn orders(&self, account_id: &AccountId) -> BackendResult<Vec<ReadOnlyOrderRecord>> {
        let value = self.client.orders(account_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn order_status(
        &self,
        account_id: &AccountId,
        broker_order_id: &str,
    ) -> BackendResult<ReadOnlyOrderRecord> {
        let orders = self.orders(account_id).await?;
        orders
            .into_iter()
            .find(|order| order.broker_order_id.as_str() == broker_order_id)
            .ok_or_else(|| {
                GatewayError::new(
                    ErrorCode::BrokerCapabilityUnavailable,
                    "Order status was not found",
                    false,
                    Some("Use a known broker order id".to_string()),
                )
            })
    }

    async fn executions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        let value = self.client.executions(account_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }
}

fn map_json_mapping_error(_error: serde_json::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerResponseInvalid,
        "Client Portal Gateway response could not be mapped safely",
        true,
        Some("Retry or inspect broker response safely".to_string()),
    )
}
