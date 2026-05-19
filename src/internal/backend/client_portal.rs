//! Client Portal Gateway backend implementation.

use super::r#trait::{BackendResult, IbkrBackend};
use crate::internal::audit::AuditHmacKey;
use crate::internal::cpapi::{
    ClientPortalClient, map_account, map_contract_candidate, map_session_response,
    map_tickle_response,
};
use crate::internal::domain::{
    AccountCapabilityProfile, AccountId, BrokerAccount, ContractCandidate, ContractId, ErrorCode,
    GatewayError, HistoricalBar, HistoricalBarsRequest, MarketSnapshot, OrdersHistory,
    OrdersHistoryRequest, PnlRealtime, PnlSnapshot, ReadOnlyOrderRecord,
};
use crate::internal::domain::{MarketDepth, OptionChain, OptionGreeks, ScannerRun};
use async_trait::async_trait;
use std::sync::Arc;

/// Broker backend backed by a local Client Portal Gateway.
#[derive(Clone)]
pub struct ClientPortalBackend {
    client: ClientPortalClient,
    audit_hmac_key: Arc<AuditHmacKey>,
}

impl ClientPortalBackend {
    /// Creates a Client Portal backend.
    #[must_use]
    pub const fn new(client: ClientPortalClient, audit_hmac_key: Arc<AuditHmacKey>) -> Self {
        Self {
            client,
            audit_hmac_key,
        }
    }
}

#[async_trait]
impl IbkrBackend for ClientPortalBackend {
    async fn session_status(&self) -> BackendResult<crate::internal::domain::BrokerSessionStatus> {
        let response = self.client.session_status().await?;
        Ok(map_session_response(response))
    }

    async fn keepalive(&self) -> BackendResult<crate::internal::domain::BrokerSessionStatus> {
        let response = self.client.tickle().await?;
        Ok(map_tickle_response(response))
    }

    async fn list_accounts(&self) -> BackendResult<Vec<BrokerAccount>> {
        let response = self.client.accounts().await?;
        response
            .accounts
            .into_iter()
            .map(|account| map_account(account, &self.audit_hmac_key))
            .collect()
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
        super::resolve_unique_contract(candidates)
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
        order_lookup_id: &str,
    ) -> BackendResult<ReadOnlyOrderRecord> {
        let orders = self.orders(account_id).await?;
        orders
            .into_iter()
            .find(|order| {
                order.broker_order_id.as_str() == order_lookup_id
                    || order.client_order_id.as_deref() == Some(order_lookup_id)
            })
            .ok_or_else(|| {
                GatewayError::new(
                    ErrorCode::BrokerCapabilityUnavailable,
                    "Order status was not found",
                    false,
                    Some("Use a known broker or client order id".to_string()),
                )
            })
    }

    async fn executions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        let value = self.client.executions(account_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn pnl_daily(&self, account_id: &AccountId) -> BackendResult<PnlSnapshot> {
        let value = self.client.pnl_daily(account_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn pnl_realtime(&self, account_id: &AccountId) -> BackendResult<PnlRealtime> {
        let value = self.client.pnl_realtime(account_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn orders_history(&self, request: &OrdersHistoryRequest) -> BackendResult<OrdersHistory> {
        let value = self
            .client
            .orders_history(
                request.account_id.as_str(),
                request.limit,
                request.from.map(|value| value.unix_timestamp()),
                request.to.map(|value| value.unix_timestamp()),
            )
            .await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn account_metadata(
        &self,
        account_id: &AccountId,
    ) -> BackendResult<AccountCapabilityProfile> {
        let value = self.client.account_metadata(account_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn options_chain(&self, symbol: &str) -> BackendResult<OptionChain> {
        let value = self.client.options_chain(symbol).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn option_greeks(&self, contract_id: &ContractId) -> BackendResult<OptionGreeks> {
        let value = self.client.option_greeks(contract_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn market_depth(&self, contract_id: &ContractId) -> BackendResult<MarketDepth> {
        let value = self.client.market_depth(contract_id.as_str()).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }

    async fn scanner_run(&self, scanner_code: &str) -> BackendResult<ScannerRun> {
        let value = self.client.scanner_run(scanner_code).await?;
        serde_json::from_value(value).map_err(map_json_mapping_error)
    }
}

fn map_json_mapping_error(error: serde_json::Error) -> GatewayError {
    // The serde error carries the JSON path and reason, which is essential for
    // diagnosing schema drift between the gateway and CPAPI. We surface it via
    // `tracing` and keep the caller-facing message stable for redaction.
    tracing::error!(
        target: "backend.client_portal",
        error = %error,
        column = error.column(),
        line = error.line(),
        "client portal gateway response could not be mapped"
    );
    GatewayError::new(
        ErrorCode::BrokerResponseInvalid,
        "Client Portal Gateway response could not be mapped safely",
        true,
        Some("Retry or inspect broker response safely".to_string()),
    )
}
