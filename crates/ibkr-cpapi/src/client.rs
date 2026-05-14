//! Minimal Client Portal Gateway HTTP client for US1 read-only calls.

use crate::models::{
    CpapiAccountsResponse, CpapiContractsResponse, CpapiExecutionsResponse,
    CpapiHistoricalBarsResponse, CpapiJsonResponse, CpapiMarketSnapshotResponse,
    CpapiOrdersResponse, CpapiSessionResponse, CpapiTickleResponse,
};
use ibkr_domain::{ErrorCode, GatewayError};
use url::Url;

/// Client Portal Gateway HTTP client.
#[derive(Clone)]
pub struct ClientPortalClient {
    base_url: Url,
    http: reqwest::Client,
}

impl ClientPortalClient {
    /// Creates a CPAPI client.
    #[must_use]
    pub fn new(base_url: Url, verify_tls: bool) -> Self {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(!verify_tls)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { base_url, http }
    }

    /// Calls the session status endpoint family.
    pub async fn session_status(&self) -> Result<CpapiSessionResponse, GatewayError> {
        self.get_json("iserver/auth/status").await
    }

    /// Calls the keepalive endpoint family.
    pub async fn tickle(&self) -> Result<CpapiTickleResponse, GatewayError> {
        self.get_json("tickle").await
    }

    /// Calls the account discovery endpoint family.
    pub async fn accounts(&self) -> Result<CpapiAccountsResponse, GatewayError> {
        self.get_json("portfolio/accounts").await
    }

    /// Calls the account summary endpoint family.
    pub async fn account_summary(
        &self,
        account_id: &str,
    ) -> Result<CpapiJsonResponse, GatewayError> {
        self.get_json(&format!("portfolio/{account_id}/summary"))
            .await
    }

    /// Calls the positions endpoint family.
    pub async fn positions(&self, account_id: &str) -> Result<CpapiJsonResponse, GatewayError> {
        self.get_json(&format!("portfolio/{account_id}/positions"))
            .await
    }

    /// Calls the portfolio snapshot endpoint family.
    pub async fn portfolio_snapshot(
        &self,
        account_id: &str,
    ) -> Result<CpapiJsonResponse, GatewayError> {
        self.get_json(&format!("portfolio/{account_id}/snapshot"))
            .await
    }

    /// Calls the contract search endpoint family.
    pub async fn contracts_search(
        &self,
        query: &str,
    ) -> Result<CpapiContractsResponse, GatewayError> {
        self.get_json(&format!("iserver/secdef/search?symbol={query}"))
            .await
    }

    /// Calls the market snapshot endpoint family.
    pub async fn market_snapshot(
        &self,
        contract_id: &str,
    ) -> Result<CpapiMarketSnapshotResponse, GatewayError> {
        self.get_json(&format!("iserver/marketdata/snapshot?conids={contract_id}"))
            .await
    }

    /// Calls the historical bars endpoint family.
    pub async fn historical_bars(
        &self,
        contract_id: &str,
        duration: &str,
        bar_size: &str,
    ) -> Result<CpapiHistoricalBarsResponse, GatewayError> {
        self.get_json(&format!(
            "iserver/marketdata/history?conid={contract_id}&period={duration}&bar={bar_size}"
        ))
        .await
    }

    /// Calls the orders endpoint family.
    pub async fn orders(&self, account_id: &str) -> Result<CpapiOrdersResponse, GatewayError> {
        self.get_json(&format!("iserver/account/{account_id}/orders"))
            .await
    }

    /// Calls the executions endpoint family.
    pub async fn executions(
        &self,
        account_id: &str,
    ) -> Result<CpapiExecutionsResponse, GatewayError> {
        self.get_json(&format!("iserver/account/{account_id}/executions"))
            .await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, GatewayError> {
        let url = self.base_url.join(path).map_err(|_| {
            GatewayError::new(
                ErrorCode::ConfigInvalid,
                "Invalid Client Portal Gateway endpoint URL",
                false,
                Some("Fix broker base URL".to_string()),
            )
        })?;

        self.http
            .get(url)
            .send()
            .await
            .map_err(map_transport_error)?
            .error_for_status()
            .map_err(map_status_error)?
            .json()
            .await
            .map_err(map_json_error)
    }
}

fn map_transport_error(_error: reqwest::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerBackendUnavailable,
        "Client Portal Gateway is unavailable",
        true,
        Some("Start or check Client Portal Gateway".to_string()),
    )
}

fn map_status_error(error: reqwest::Error) -> GatewayError {
    if error.status().is_some_and(|status| status.as_u16() == 401) {
        GatewayError::new(
            ErrorCode::BrokerSessionRequired,
            "Broker session requires manual authentication",
            true,
            Some("Complete broker login manually".to_string()),
        )
    } else {
        GatewayError::new(
            ErrorCode::BrokerBackendUnavailable,
            "Client Portal Gateway returned an unavailable status",
            true,
            Some("Check Client Portal Gateway status".to_string()),
        )
    }
}

fn map_json_error(_error: reqwest::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerResponseInvalid,
        "Client Portal Gateway response could not be mapped safely",
        true,
        Some("Retry or inspect broker response safely".to_string()),
    )
}
