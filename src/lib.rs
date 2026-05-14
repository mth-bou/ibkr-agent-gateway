//! Test harness package for top-level contract and integration tests.
//!
//! Production logic lives in the workspace crates under `crates/`.

/// Marker used by integration tests to prove the harness is available.
pub const HARNESS_NAME: &str = "ibkr-agent-gateway";
