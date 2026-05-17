//! Preview-only order services.
//!
//! This crate intentionally has no broker submit, cancel, approve, or live
//! execution path of its own — broker-side calls are delegated to a
//! writer implementation supplied by the deployment.

mod approval_gate;
pub mod audit;
pub mod idempotency;
pub mod kill_switch;
pub mod lifecycle;
pub mod live_cancel;
pub mod live_migration;
pub mod live_submit;
pub mod live_writer;
pub mod paper_cancel;
pub mod paper_submit;
pub mod paper_writer;
pub mod preview;
pub mod recovery;
pub mod validated_order;

pub use crate::internal::domain::{OrderIntent, OrderPreview, ValidatedOrder};
pub use audit::build_order_audit_event;
pub use idempotency::{
    IdempotencyDecision, IdempotencyKey, IdempotencyRecord, IdempotencyStore, stable_request_hash,
};
pub use kill_switch::{KillSwitch, KillSwitchState, KillSwitchStore};
pub use lifecycle::{
    LiveExecutionCorrelation, LiveOrderLifecycleRecord, LiveOrderLifecycleStatus,
    PaperOrderLifecycleRecord, PaperOrderLifecycleStatus,
};
pub use live_cancel::{LiveCancelRequest, LiveCancelResult, cancel_live_order};
pub use live_migration::{PaperToLiveMigrationChecklist, validate_paper_to_live_migration};
pub use live_submit::{LiveSubmitRequest, LiveSubmitResult, submit_live_order};
pub use live_writer::{
    LiveCancelReceipt, LiveOrderWriter, LiveSubmitReceipt, LocalCandidateLiveWriter,
    RefusingLiveWriter,
};
pub use paper_cancel::{PaperCancelRequest, PaperCancelResult, cancel_paper_order};
pub use paper_submit::{PaperSubmitRequest, PaperSubmitResult, submit_paper_order};
pub use paper_writer::{
    LocalCandidatePaperWriter, PaperCancelReceipt, PaperOrderWriter, PaperSubmitReceipt,
    RefusingPaperWriter,
};
pub use preview::create_order_preview;
pub use recovery::{OrderIdempotencyRecoveryReport, recover_pending_order_idempotency};
pub use validated_order::build_validated_order;
