//! Backend abstraction crate for broker implementations and fake fixtures.

pub mod fake;
pub mod r#trait;

pub use fake::FakeFixtureStore;
pub use r#trait::{BackendResult, IbkrBackend};
