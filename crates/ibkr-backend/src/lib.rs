//! Backend abstraction crate for broker implementations and fake fixtures.

pub mod client_portal;
pub mod factory;
pub mod fake;
pub mod r#trait;

pub use client_portal::ClientPortalBackend;
pub use factory::{BackendFactoryConfig, create_backend};
pub use fake::{FakeBackend, FakeFixtureStore};
pub use r#trait::{BackendResult, IbkrBackend};
