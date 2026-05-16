//! Fake backend fixture loading support.

use super::r#trait::{BackendResult, IbkrBackend};
use crate::internal::domain::{
    AccountId, BrokerAccount, BrokerSessionStatus, ContractCandidate, ContractId, HistoricalBar,
    HistoricalBarsRequest, MarketSnapshot, ReadOnlyOrderRecord,
};
use crate::internal::domain::{ErrorCode, GatewayError};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

/// Filesystem-backed fake broker fixtures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeFixtureStore {
    root: PathBuf,
}

impl FakeFixtureStore {
    /// Creates a fixture store rooted at `root`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the fixture root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Loads and deserializes one JSON fixture.
    pub fn load_json<T: DeserializeOwned>(&self, relative_path: &str) -> Result<T, GatewayError> {
        let path = self.root.join(relative_path);
        let raw = std::fs::read_to_string(&path).map_err(|_| {
            GatewayError::new(
                ErrorCode::BrokerBackendUnavailable,
                format!("Missing fake backend fixture: {}", path.display()),
                true,
                Some("Create the requested fake backend fixture".to_string()),
            )
        })?;

        serde_json::from_str(&raw).map_err(|_| {
            GatewayError::new(
                ErrorCode::BrokerResponseInvalid,
                format!("Invalid fake backend fixture JSON: {}", path.display()),
                true,
                Some("Fix the fake backend fixture JSON".to_string()),
            )
        })
    }
}

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
        self.fixtures.load_json("session_status_usable.json")
    }

    async fn keepalive(&self) -> BackendResult<BrokerSessionStatus> {
        self.fixtures.load_json("tickle_success.json")
    }

    async fn list_accounts(&self) -> BackendResult<Vec<BrokerAccount>> {
        self.fixtures.load_json("accounts_success.json")
    }

    async fn account_summary(&self, account_id: &AccountId) -> BackendResult<serde_json::Value> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("portfolio_snapshot.json")
    }

    async fn portfolio_snapshot(&self, account_id: &AccountId) -> BackendResult<serde_json::Value> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("portfolio_snapshot.json")
    }

    async fn positions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("positions_list.json")
    }

    async fn search_contracts(&self, query: &str) -> BackendResult<Vec<ContractCandidate>> {
        if query.eq_ignore_ascii_case("AMBIG") {
            return self.fixtures.load_json("contracts_ambiguous.json");
        }
        self.fixtures.load_json("contracts_search_stock_etf.json")
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

    async fn market_snapshot(&self, _contract_id: &ContractId) -> BackendResult<MarketSnapshot> {
        self.fixtures.load_json("market_snapshot_live.json")
    }

    async fn historical_bars(
        &self,
        _request: &HistoricalBarsRequest,
    ) -> BackendResult<Vec<HistoricalBar>> {
        self.fixtures.load_json("historical_bars.json")
    }

    async fn orders(&self, account_id: &AccountId) -> BackendResult<Vec<ReadOnlyOrderRecord>> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("orders_list.json")
    }

    async fn order_status(
        &self,
        account_id: &AccountId,
        _broker_order_id: &str,
    ) -> BackendResult<ReadOnlyOrderRecord> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("order_status.json")
    }

    async fn executions(&self, account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        validate_account_id(account_id)?;
        self.fixtures.load_json("executions_list.json")
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
