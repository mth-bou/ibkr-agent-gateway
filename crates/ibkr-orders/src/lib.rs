//! Preview-only order services.
//!
//! This crate intentionally has no broker submit, cancel, approve, or live
//! execution path.

pub mod preview;
pub mod validated_order;

pub use ibkr_domain::{OrderIntent, OrderPreview, ValidatedOrder};
pub use preview::create_order_preview;
pub use validated_order::build_validated_order;
