//! Preview-only order services.
//!
//! This crate intentionally has no broker submit, cancel, approve, or live
//! execution path.

pub mod audit;
pub mod idempotency;
pub mod preview;
pub mod validated_order;

pub use audit::build_order_audit_event;
pub use ibkr_domain::{OrderIntent, OrderPreview, ValidatedOrder};
pub use idempotency::{IdempotencyKey, IdempotencyRecord};
pub use preview::create_order_preview;
pub use validated_order::build_validated_order;
