//! Client Portal Gateway HTTP client.

use super::models::{
    CpapiAccountsResponse, CpapiContractsResponse, CpapiExecutionsResponse,
    CpapiHistoricalBarsResponse, CpapiJsonResponse, CpapiMarketSnapshotResponse,
    CpapiOrdersResponse, CpapiSessionResponse, CpapiTickleResponse,
};
use crate::internal::domain::{ErrorCode, GatewayError};
use std::time::Duration as StdDuration;
use url::Url;

const DEFAULT_HTTP_TIMEOUT: StdDuration = StdDuration::from_secs(10);
const DEFAULT_MAX_BODY_BYTES: usize = 1024 * 1024;

/// Client Portal Gateway HTTP client.
#[derive(Clone)]
pub struct ClientPortalClient {
    base_url: Url,
    http: reqwest::Client,
    max_body_bytes: usize,
}

impl ClientPortalClient {
    /// Creates a CPAPI client.
    pub fn new(base_url: Url, verify_tls: bool) -> Result<Self, GatewayError> {
        Self::with_timeout(base_url, verify_tls, DEFAULT_HTTP_TIMEOUT)
    }

    /// Creates a CPAPI client with an explicit request and connect timeout.
    pub fn with_timeout(
        base_url: Url,
        verify_tls: bool,
        timeout: StdDuration,
    ) -> Result<Self, GatewayError> {
        Self::with_limits(base_url, verify_tls, timeout, DEFAULT_MAX_BODY_BYTES)
    }

    /// Creates a CPAPI client with explicit timeout and response body limit.
    pub fn with_limits(
        base_url: Url,
        verify_tls: bool,
        timeout: StdDuration,
        max_body_bytes: usize,
    ) -> Result<Self, GatewayError> {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(!verify_tls)
            .timeout(timeout)
            .connect_timeout(timeout)
            .build()
            .map_err(|_| {
                GatewayError::new(
                    ErrorCode::ConfigInvalid,
                    "Unable to initialize Client Portal Gateway HTTP client",
                    true,
                    Some("Check Client Portal Gateway HTTP client configuration".to_string()),
                )
            })?;
        Ok(Self {
            base_url,
            http,
            max_body_bytes,
        })
    }

    /// Calls the session status endpoint family.
    pub async fn session_status(&self) -> Result<CpapiSessionResponse, GatewayError> {
        self.get_json(&["iserver", "auth", "status"], &[]).await
    }

    /// Calls the keepalive endpoint family.
    pub async fn tickle(&self) -> Result<CpapiTickleResponse, GatewayError> {
        self.get_json(&["tickle"], &[]).await
    }

    /// Calls the account discovery endpoint family.
    pub async fn accounts(&self) -> Result<CpapiAccountsResponse, GatewayError> {
        self.get_json(&["portfolio", "accounts"], &[]).await
    }

    /// Calls the account summary endpoint family.
    pub async fn account_summary(
        &self,
        account_id: &str,
    ) -> Result<CpapiJsonResponse, GatewayError> {
        self.get_json(&["portfolio", account_id, "summary"], &[])
            .await
    }

    /// Calls the positions endpoint family.
    pub async fn positions(&self, account_id: &str) -> Result<CpapiJsonResponse, GatewayError> {
        self.get_json(&["portfolio", account_id, "positions"], &[])
            .await
    }

    /// Calls the portfolio snapshot endpoint family.
    pub async fn portfolio_snapshot(
        &self,
        account_id: &str,
    ) -> Result<CpapiJsonResponse, GatewayError> {
        self.get_json(&["portfolio", account_id, "snapshot"], &[])
            .await
    }

    /// Calls the contract search endpoint family.
    pub async fn contracts_search(
        &self,
        query: &str,
    ) -> Result<CpapiContractsResponse, GatewayError> {
        self.get_json(&["iserver", "secdef", "search"], &[("symbol", query)])
            .await
    }

    /// Calls the market snapshot endpoint family.
    pub async fn market_snapshot(
        &self,
        contract_id: &str,
    ) -> Result<CpapiMarketSnapshotResponse, GatewayError> {
        self.get_json(
            &["iserver", "marketdata", "snapshot"],
            &[("conids", contract_id)],
        )
        .await
    }

    /// Calls the historical bars endpoint family.
    pub async fn historical_bars(
        &self,
        contract_id: &str,
        duration: &str,
        bar_size: &str,
    ) -> Result<CpapiHistoricalBarsResponse, GatewayError> {
        self.get_json(
            &["iserver", "marketdata", "history"],
            &[
                ("conid", contract_id),
                ("period", duration),
                ("bar", bar_size),
            ],
        )
        .await
    }

    /// Calls the orders endpoint family.
    pub async fn orders(&self, account_id: &str) -> Result<CpapiOrdersResponse, GatewayError> {
        self.get_json(&["iserver", "account", account_id, "orders"], &[])
            .await
    }

    /// Calls the executions endpoint family.
    pub async fn executions(
        &self,
        account_id: &str,
    ) -> Result<CpapiExecutionsResponse, GatewayError> {
        self.get_json(&["iserver", "account", account_id, "executions"], &[])
            .await
    }

    /// Posts a JSON body to a path and decodes the JSON response.
    pub async fn post_json<T, B>(&self, path_segments: &[&str], body: &B) -> Result<T, GatewayError>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let url = self.endpoint(path_segments, &[])?;
        let request = self
            .http
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "application/json")
            .json(body);
        self.send_json(request).await
    }

    /// Deletes a path and decodes the JSON response.
    pub async fn delete_json<T>(&self, path_segments: &[&str]) -> Result<T, GatewayError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.endpoint(path_segments, &[])?;
        let request = self
            .http
            .delete(url)
            .header(reqwest::header::ACCEPT, "application/json");
        self.send_json(request).await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path_segments: &[&str],
        query_pairs: &[(&str, &str)],
    ) -> Result<T, GatewayError> {
        let url = self.endpoint(path_segments, query_pairs)?;

        self.get_json_url(url).await
    }

    async fn get_json_url<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
    ) -> Result<T, GatewayError> {
        let request = self.http.get(url);
        self.send_json(request).await
    }

    async fn send_json<T: serde::de::DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, GatewayError> {
        let mut response = request
            .send()
            .await
            .map_err(map_transport_error)?
            .error_for_status()
            .map_err(map_status_error)?;

        if response
            .content_length()
            .is_some_and(|length| length > self.max_body_bytes as u64)
        {
            return Err(map_body_too_large());
        }

        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(map_transport_error)? {
            if body.len().saturating_add(chunk.len()) > self.max_body_bytes {
                return Err(map_body_too_large());
            }
            body.extend_from_slice(&chunk);
        }

        serde_json::from_slice(&body).map_err(map_json_error)
    }

    fn endpoint(
        &self,
        path_segments: &[&str],
        query_pairs: &[(&str, &str)],
    ) -> Result<Url, GatewayError> {
        let mut url = self.base_url.clone();
        url.set_query(None);
        url.set_fragment(None);
        {
            let mut segments = url.path_segments_mut().map_err(|_| invalid_endpoint())?;
            segments.pop_if_empty();
            for segment in path_segments {
                segments.push(validate_path_segment(segment)?);
            }
        }
        if !query_pairs.is_empty() {
            let mut query = url.query_pairs_mut();
            for (key, value) in query_pairs {
                query.append_pair(validate_query_key(key)?, validate_query_value(value)?);
            }
        }
        Ok(url)
    }
}

fn validate_path_segment(value: &str) -> Result<&str, GatewayError> {
    if value.is_empty()
        || value.trim() != value
        || value
            .chars()
            .any(|ch| ch.is_ascii_control() || matches!(ch, '/' | '?' | '#'))
    {
        return Err(invalid_endpoint());
    }
    Ok(value)
}

fn validate_query_key(value: &str) -> Result<&str, GatewayError> {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_ascii_control() || matches!(ch, '&' | '=' | '?' | '#'))
    {
        return Err(invalid_endpoint());
    }
    Ok(value)
}

fn validate_query_value(value: &str) -> Result<&str, GatewayError> {
    if value.is_empty() || value.chars().any(|ch| ch.is_ascii_control()) {
        return Err(invalid_endpoint());
    }
    Ok(value)
}

fn invalid_endpoint() -> GatewayError {
    GatewayError::new(
        ErrorCode::ConfigInvalid,
        "Invalid Client Portal Gateway endpoint URL",
        false,
        Some("Use valid broker identifiers and base URL".to_string()),
    )
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

fn map_json_error(_error: serde_json::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerResponseInvalid,
        "Client Portal Gateway response could not be mapped safely",
        true,
        Some("Retry or inspect broker response safely".to_string()),
    )
}

fn map_body_too_large() -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerResponseInvalid,
        "Client Portal Gateway response exceeded the configured size limit",
        true,
        Some("Retry or inspect broker response size safely".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::ClientPortalClient;
    use url::Url;

    fn client() -> Result<ClientPortalClient, Box<dyn std::error::Error>> {
        Ok(ClientPortalClient::new(
            Url::parse("https://localhost:5000/v1/api/")?,
            true,
        )?)
    }

    #[test]
    fn endpoint_encodes_query_values_without_raw_interpolation()
    -> Result<(), Box<dyn std::error::Error>> {
        let url = client()?.endpoint(
            &["iserver", "marketdata", "history"],
            &[("conid", "265598"), ("period", "1 D"), ("bar", "5 mins")],
        )?;

        assert_eq!(
            url.as_str(),
            "https://localhost:5000/v1/api/iserver/marketdata/history?conid=265598&period=1+D&bar=5+mins"
        );
        Ok(())
    }

    #[test]
    fn endpoint_rejects_path_separator_in_path_segments() -> Result<(), Box<dyn std::error::Error>>
    {
        let error = client()?.endpoint(&["portfolio", "DU123/../other", "summary"], &[]);
        let Err(error) = error else {
            return Err("path traversal-like account ids should be rejected".into());
        };

        assert!(error.message.contains("Invalid Client Portal Gateway"));
        Ok(())
    }

    #[test]
    fn endpoint_rejects_empty_or_control_query_values() -> Result<(), Box<dyn std::error::Error>> {
        let empty = client()?.endpoint(&["iserver", "secdef", "search"], &[("symbol", "")]);
        let Err(empty) = empty else {
            return Err("empty query values should be rejected".into());
        };
        let control = client()?.endpoint(
            &["iserver", "secdef", "search"],
            &[("symbol", "AAPL\nMSFT")],
        );
        let Err(control) = control else {
            return Err("control characters should be rejected".into());
        };

        assert_eq!(empty.code, control.code);
        Ok(())
    }
}
