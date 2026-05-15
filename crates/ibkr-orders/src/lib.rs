//! Preview-only order services.
//!
//! This crate intentionally has no broker submit, cancel, approve, or live
//! execution path.

pub mod audit;
pub mod idempotency;
pub mod kill_switch;
pub mod lifecycle;
pub mod live_cancel;
pub mod live_migration;
pub mod live_submit;
pub mod paper_cancel;
pub mod paper_submit;
pub mod preview;
pub mod validated_order;

pub use audit::build_order_audit_event;
pub use ibkr_domain::{OrderIntent, OrderPreview, ValidatedOrder};
pub use idempotency::{IdempotencyDecision, IdempotencyKey, IdempotencyRecord, IdempotencyStore};
pub use kill_switch::{KillSwitch, KillSwitchState, KillSwitchStore};
pub use lifecycle::{
    LiveExecutionCorrelation, LiveOrderLifecycleRecord, LiveOrderLifecycleStatus,
    PaperOrderLifecycleRecord, PaperOrderLifecycleStatus,
};
pub use live_cancel::{LiveCancelRequest, LiveCancelResult, cancel_live_order};
pub use live_migration::{PaperToLiveMigrationChecklist, validate_paper_to_live_migration};
pub use live_submit::{LiveSubmitRequest, LiveSubmitResult, submit_live_order};
pub use paper_cancel::{PaperCancelRequest, PaperCancelResult, cancel_paper_order};
pub use paper_submit::{PaperSubmitRequest, PaperSubmitResult, submit_paper_order};
pub use preview::create_order_preview;
pub use validated_order::build_validated_order;
