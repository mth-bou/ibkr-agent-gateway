// All internal items below are reachable via the `testing` module when the
// `unstable-internal-test-support` feature is enabled. Without the feature,
// many helpers used only by integration tests look unused to the compiler —
// silence those *only* in that build configuration so production drift remains
// hidden, but developers running `cargo test --features
// unstable-internal-test-support` (the CI gate) still see real dead-code
// warnings on actual orphans.
#![cfg_attr(
    not(feature = "unstable-internal-test-support"),
    allow(dead_code, unused_imports)
)]

pub(crate) mod approval;
pub(crate) mod audit;
pub(crate) mod auth;
pub(crate) mod backend;
pub(crate) mod config;
pub(crate) mod cpapi;
pub(crate) mod domain;
pub(crate) mod mcp;
pub(crate) mod oauth;
pub(crate) mod observability;
pub(crate) mod orders;
pub(crate) mod provider_compat;
pub(crate) mod risk;
pub(crate) mod sidecar;
