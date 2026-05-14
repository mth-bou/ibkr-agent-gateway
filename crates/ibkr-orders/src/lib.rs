//! Preview-only order services.
//!
//! This crate intentionally has no broker submit, cancel, approve, or live
//! execution path.

pub use ibkr_domain::{OrderIntent, OrderPreview, ValidatedOrder};
