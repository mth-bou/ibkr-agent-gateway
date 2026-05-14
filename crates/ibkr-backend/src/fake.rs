//! Fake backend fixture loading support.

use crate::r#trait::{BackendResult, IbkrBackend};
use async_trait::async_trait;
use ibkr_domain::{
    AccountId, BrokerAccount, BrokerSessionStatus, ContractCandidate, ContractId, HistoricalBar,
    HistoricalBarsRequest, MarketSnapshot, ReadOnlyOrderRecord,
};
use ibkr_domain::{ErrorCode, GatewayError};
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

    async fn account_summary(&self, _account_id: &AccountId) -> BackendResult<serde_json::Value> {
        Err(capability_unavailable())
    }

    async fn positions(&self, _account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        Err(capability_unavailable())
    }

    async fn search_contracts(&self, _query: &str) -> BackendResult<Vec<ContractCandidate>> {
        Err(capability_unavailable())
    }

    async fn resolve_contract(&self, _query: &str) -> BackendResult<ContractCandidate> {
        Err(capability_unavailable())
    }

    async fn market_snapshot(&self, _contract_id: &ContractId) -> BackendResult<MarketSnapshot> {
        Err(capability_unavailable())
    }

    async fn historical_bars(
        &self,
        _request: &HistoricalBarsRequest,
    ) -> BackendResult<Vec<HistoricalBar>> {
        Err(capability_unavailable())
    }

    async fn orders(&self, _account_id: &AccountId) -> BackendResult<Vec<ReadOnlyOrderRecord>> {
        Err(capability_unavailable())
    }

    async fn order_status(
        &self,
        _account_id: &AccountId,
        _broker_order_id: &str,
    ) -> BackendResult<ReadOnlyOrderRecord> {
        Err(capability_unavailable())
    }

    async fn executions(&self, _account_id: &AccountId) -> BackendResult<Vec<serde_json::Value>> {
        Err(capability_unavailable())
    }
}

fn capability_unavailable() -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerCapabilityUnavailable,
        "Capability is not implemented in this fake backend slice",
        false,
        Some("Use a later read-only feature task".to_string()),
    )
}
