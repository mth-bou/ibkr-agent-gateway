//! Fake backend fixture loading support.

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
