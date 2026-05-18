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
use std::path::{Component, Path, PathBuf};
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
                format!("Invalid fake backend fixture JSON shape: {relative_path}"),
                true,
                Some("Fix the fake backend fixture JSON".to_string()),
            )
        })
    }

    async fn load_value(
        &self,
        relative_path: &str,
    ) -> Result<Arc<serde_json::Value>, GatewayError> {
        if let Some(value) = self.cached(relative_path) {
            return Ok(value);
        }

        let path = safe_join(&self.root, relative_path)?;
        let raw = tokio::fs::read_to_string(&path).await.map_err(|err| {
            tracing::error!(
                target: "backend.fake",
                path = %path.display(),
                error = %err,
                error_kind = ?err.kind(),
                "fake backend fixture is missing or unreadable"
            );
            GatewayError::new(
                ErrorCode::BrokerBackendUnavailable,
                format!("Missing fake backend fixture: {}", path.display()),
                true,
                Some("Create the requested fake backend fixture".to_string()),
            )
        })?;

        let value = serde_json::from_str::<serde_json::Value>(&raw).map_err(|err| {
            tracing::error!(
                target: "backend.fake",
                path = %path.display(),
                error = %err,
                line = err.line(),
                column = err.column(),
                "fake backend fixture JSON is invalid"
            );
            GatewayError::new(
                ErrorCode::BrokerResponseInvalid,
                format!("Invalid fake backend fixture JSON: {}", path.display()),
                true,
                Some("Fix the fake backend fixture JSON".to_string()),
            )
        })?;
        let value = Arc::new(value);
        self.cache_value(relative_path, value.clone());
        Ok(value)
    }

    fn cached(&self, relative_path: &str) -> Option<Arc<serde_json::Value>> {
        let cache = self
            .cache
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.get(relative_path).cloned()
    }

    fn cache_value(&self, relative_path: &str, value: Arc<serde_json::Value>) {
        let mut cache = self
            .cache
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        cache.insert(relative_path.to_string(), value);
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
        super::resolve_unique_contract(candidates)
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
        _order_lookup_id: &str,
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

/// Joins a fixture-relative path onto `root`, refusing absolute paths,
/// drive-relative paths, and any path component that would escape the root.
///
/// Mitigates review finding H-2 (2026-05-17): the fixture root is supplied via
/// configuration and may contain attacker-controlled relative segments.
fn safe_join(root: &Path, relative_path: &str) -> Result<PathBuf, GatewayError> {
    let candidate = Path::new(relative_path);
    for component in candidate.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(GatewayError::new(
                    ErrorCode::BrokerResponseInvalid,
                    "Fake backend fixture path must be relative and within the root",
                    false,
                    Some(
                        "Use a fixture name without absolute prefixes or parent segments"
                            .to_string(),
                    ),
                ));
            }
        }
    }
    Ok(root.join(candidate))
}

#[cfg(test)]
mod tests {
    use super::safe_join;
    use crate::internal::domain::ErrorCode;
    use std::path::Path;

    #[test]
    fn rejects_parent_dir_components() {
        let Err(error) = safe_join(Path::new("/srv/fixtures"), "../../etc/passwd") else {
            unreachable!("parent-dir traversal must be rejected");
        };
        assert_eq!(error.code, ErrorCode::BrokerResponseInvalid);
    }

    #[test]
    fn rejects_absolute_relative_path() {
        let Err(error) = safe_join(Path::new("/srv/fixtures"), "/etc/passwd") else {
            unreachable!("absolute relative path must be rejected");
        };
        assert_eq!(error.code, ErrorCode::BrokerResponseInvalid);
    }

    #[test]
    fn accepts_simple_filename() {
        let joined = safe_join(Path::new("/srv/fixtures"), "accounts_success.json");
        assert!(joined.is_ok());
    }

    #[test]
    fn accepts_nested_normal_components() {
        let joined = safe_join(Path::new("/srv/fixtures"), "v1/accounts.json");
        assert!(joined.is_ok());
    }

    #[test]
    fn rejects_embedded_parent_segment() {
        let Err(error) = safe_join(Path::new("/srv/fixtures"), "v1/../../../etc/passwd") else {
            unreachable!("embedded traversal must be rejected");
        };
        assert_eq!(error.code, ErrorCode::BrokerResponseInvalid);
    }

    #[test]
    #[allow(clippy::panic)] // Test deliberately poisons the lock via panic.
    fn fixture_cache_survives_lock_poisoning() {
        use super::FakeFixtureStore;
        use std::sync::Arc;
        use std::thread;

        let store = FakeFixtureStore::new("/tmp/ibkr-agent-gateway/fixtures");

        // Pre-populate the cache so we can observe survival across the poison.
        store.cache_value("pre-poison.json", Arc::new(serde_json::Value::Bool(true)));

        // Deliberately poison the RwLock by panicking while holding the
        // write guard from another thread. `JoinHandle::join` returns `Err`
        // when the thread panicked; we ignore that.
        let poisoner_store = store.clone();
        let _ = thread::spawn(move || {
            let _guard = poisoner_store
                .cache
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            panic!("intentional poison for test");
        })
        .join();

        // Both read and write paths must still work after poisoning.
        let cached = store.cached("pre-poison.json");
        assert!(cached.is_some(), "cached entry must survive poison");

        store.cache_value("post-poison.json", Arc::new(serde_json::Value::Bool(false)));
        let cached_after = store.cached("post-poison.json");
        assert!(cached_after.is_some(), "writes must succeed after poison");
    }
}
