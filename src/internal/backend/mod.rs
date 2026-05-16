//! Backend abstraction crate for broker implementations and fake fixtures.

pub mod account_context;
pub mod client_portal;
pub mod factory;
pub mod fake;
pub mod market_data_policy;
pub mod r#trait;

pub use account_context::require_account_id;
pub use client_portal::ClientPortalBackend;
pub use factory::{BackendFactoryConfig, create_backend};
pub use fake::{FakeBackend, FakeFixtureStore};
pub use market_data_policy::apply_market_data_policy;
pub use r#trait::{BackendResult, IbkrBackend};
