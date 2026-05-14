//! Preview-only order services.
//!
//! This crate intentionally has no broker submit, cancel, approve, or live
//! execution path.

pub mod audit;
pub mod idempotency;
pub mod lifecycle;
pub mod paper_cancel;
pub mod paper_submit;
pub mod preview;
pub mod validated_order;

pub use audit::build_order_audit_event;
pub use ibkr_domain::{OrderIntent, OrderPreview, ValidatedOrder};
pub use idempotency::{IdempotencyKey, IdempotencyRecord};
pub use lifecycle::{PaperOrderLifecycleRecord, PaperOrderLifecycleStatus};
pub use paper_cancel::{PaperCancelRequest, PaperCancelResult, cancel_paper_order};
pub use paper_submit::{PaperSubmitRequest, PaperSubmitResult, submit_paper_order};
pub use preview::create_order_preview;
pub use validated_order::build_validated_order;
