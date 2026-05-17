//! Fake backend fixture loading support.

use super::r#trait::{BackendResult, IbkrBackend};
use crate::internal::domain::{
    AccountId, BrokerAccount, BrokerSessionStatus, ContractCandidate, ContractId, HistoricalBar,
    HistoricalBarsRequest, MarketSnapshot, ReadOnlyOrderRecord,
};
use crate::internal::domain::{ErrorCode, GatewayError};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Filesystem-backed fake broker fixtures.
#[derive(Clone, Debug)]
pub struct FakeFixtureStore {
    root: PathBuf,
    cache: Arc<RwLock<BTreeMap<String, Arc<serde_json::Value>>>>,
}

impl FakeFixtureStore {
    /// Creates a fixture store rooted at `root`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Returns the fixture root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Loads and deserializes one JSON fixture.
    pub async fn load_json<T: DeserializeOwned>(
        &self,
        relative_path: &str,
    ) -> Result<T, GatewayError> {
        let value = self.load_value(relative_path).await?;
        serde_json::from_value((*value).clone()).map_err(|_| {
            GatewayError::new(
                ErrorCode::BrokerResponseInvalid,
                format!(
                    "Invalid fake backend fixture JSON shape: {}",
                    self.root.join(relative_path).display()
                ),
                true,
                Some("Fix the fake backend fixture JSON".to_string()),
            )
        })
    }

    async fn load_value(
        &self,
        relative_path: &str,
    ) -> Result<Arc<serde_json::Value>, GatewayError> {
        if let Some(value) = self.cached(relative_path)? {
            return Ok(value);
        }

        let path = self.root.join(relative_path);
        let raw = tokio::fs::read_to_string(&path).await.map_err(|_| {
            GatewayError::new(
                ErrorCode::BrokerBackendUnavailable,
                format!("Missing fake backend fixture: {}", path.display()),
                true,
                Some("Create the requested fake backend fixture".to_string()),
            )
        })?;

        let value = serde_json::from_str::<serde_json::Value>(&raw).map_err(|_| {
            GatewayError::new(
                ErrorCode::BrokerResponseInvalid,
                format!("Invalid fake backend fixture JSON: {}", path.display()),
                true,
                Some("Fix the fake backend fixture JSON".to_string()),
            )
        })?;
        let value = Arc::new(value);
        self.cache_value(relative_path, value.clone())?;
        Ok(value)
    }

    fn cached(&self, relative_path: &str) -> Result<Option<Arc<serde_json::Value>>, GatewayError> {
        let cache = self.cache.read().map_err(|_| fixture_cache_error())?;
        Ok(cache.get(relative_path).cloned())
    }

    fn cache_value(
        &self,
        relative_path: &str,
        value: Arc<serde_json::Value>,
    ) -> Result<(), GatewayError> {
        let mut cache = self.cache.write().map_err(|_| fixture_cache_error())?;
        cache.insert(relative_path.to_string(), value);
        Ok(())
    }
}

impl PartialEq for FakeFixtureStore {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}

impl Eq for FakeFixtureStore {}

/// Fake backend for offline validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeBackend {
    fixtures: FakeFixtureStore,
}

impl FakeBackend {
    /// Creates a fake backend rooted at `fixtures`.
    #[must_use]
    pub const fn new(fixtures: FakeFixtureStore) -> Self {
        Self { fixtures }
    }
}

#[async_trait]
impl IbkrBackend for FakeBackend {
    async fn session_status(&self) -> BackendResult<BrokerSessionStatus> {
        self.fixtures.load_json("session_status_usable.json").await
    }

    async fn keepalive(&self) -> BackendResult<BrokerSessionStatus> {
        self.fixtures.load_json("tickle_success.json").await
    }

    async fn list_accounts(&self) -> BackendResult<Vec<BrokerAccount>> {
        self.fixtures.load_json("accounts_success.json").await
    }

    async fn account_summary(&self, account_id: &AccountId) -> BackendResult<serde_json::Value> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("portfolio_snapshot.json").await
    }

    async fn portfolio_snapshot(&self, account_id: &AccountId) -> BackendResult<serde_json::Value> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("portfolio_snapshot.json").await
    }

    async fn positions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("positions_list.json").await
    }

    async fn search_contracts(&self, query: &str) -> BackendResult<Vec<ContractCandidate>> {
        if query.eq_ignore_ascii_case("AMBIG") {
            return self.fixtures.load_json("contracts_ambiguous.json").await;
        }
        self.fixtures
            .load_json("contracts_search_stock_etf.json")
            .await
    }

    async fn resolve_contract(&self, query: &str) -> BackendResult<ContractCandidate> {
        let candidates = self.search_contracts(query).await?;
        let mut unique = None;
        for candidate in candidates {
            if !candidate.is_unique_match {
                continue;
            }
            if unique.replace(candidate).is_some() {
                return Err(GatewayError::new(
                    ErrorCode::InputAmbiguousContract,
                    "Contract resolution is ambiguous",
                    false,
                    Some("Provide symbol, asset class, currency, and exchange".to_string()),
                ));
            }
        }
        unique.ok_or_else(|| {
            GatewayError::new(
                ErrorCode::InputAmbiguousContract,
                "Contract resolution is ambiguous",
                false,
                Some("Provide symbol, asset class, currency, and exchange".to_string()),
            )
        })
    }

    async fn market_snapshot(&self, _contract_id: &ContractId) -> BackendResult<MarketSnapshot> {
        self.fixtures.load_json("market_snapshot_live.json").await
    }

    async fn historical_bars(
        &self,
        _request: &HistoricalBarsRequest,
    ) -> BackendResult<Vec<HistoricalBar>> {
        self.fixtures.load_json("historical_bars.json").await
    }

    async fn orders(&self, account_id: &AccountId) -> BackendResult<Vec<ReadOnlyOrderRecord>> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("orders_list.json").await
    }

    async fn order_status(
        &self,
        account_id: &AccountId,
        _broker_order_id: &str,
    ) -> BackendResult<ReadOnlyOrderRecord> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("order_status.json").await
    }

    async fn executions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("executions_list.json").await
    }
}

fn validate_account_id(account_id: &AccountId) -> Result<(), GatewayError> {
    if account_id.as_str().is_empty() {
        Err(GatewayError::new(
            ErrorCode::InputMissingAccount,
            "Account id is required",
            false,
            Some("Select one account explicitly".to_string()),
        ))
    } else {
        Ok(())
    }
}

fn fixture_cache_error() -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerBackendUnavailable,
        "Fake backend fixture cache is unavailable",
        true,
        Some("Retry the fake backend request".to_string()),
    )
}
