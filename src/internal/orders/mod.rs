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
pub mod live_modify;
pub mod live_submit;
pub mod live_writer;
pub mod order_modify;
pub mod paper_cancel;
pub mod paper_modify;
pub mod paper_submit;
pub mod paper_writer;
pub mod pending;
pub mod preview;
pub mod reconciler;
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
pub(crate) use live_cancel::cancel_live_order_without_local_idempotency;
pub use live_cancel::{LiveCancelRequest, LiveCancelResult, cancel_live_order};
pub use live_migration::{PaperToLiveMigrationChecklist, validate_paper_to_live_migration};
pub(crate) use live_modify::modify_live_order_without_local_idempotency;
pub use live_modify::{LiveModifyRequest, LiveModifyResult, modify_live_order};
pub(crate) use live_submit::submit_live_order_without_local_idempotency;
pub use live_submit::{LiveSubmitRequest, LiveSubmitResult, submit_live_order};
pub use live_writer::{
    LiveCancelReceipt, LiveModifyReceipt, LiveOrderWriter, LiveSubmitReceipt,
    LocalCandidateLiveWriter, RefusingLiveWriter,
};
pub use order_modify::OrderModifyFields;
pub use paper_cancel::{PaperCancelRequest, PaperCancelResult, cancel_paper_order};
pub use paper_modify::{PaperModifyRequest, PaperModifyResult, modify_paper_order};
pub use paper_submit::{PaperSubmitRequest, PaperSubmitResult, submit_paper_order};
pub use paper_writer::{
    LocalCandidatePaperWriter, PaperCancelReceipt, PaperModifyReceipt, PaperOrderWriter,
    PaperSubmitReceipt, RefusingPaperWriter,
};
pub(crate) use pending::handle_pending_order_error;
pub use preview::create_order_preview;
pub use reconciler::{LiveOrderReconciliationReport, reconcile_live_orders_once};
pub use recovery::{OrderIdempotencyRecoveryReport, recover_pending_order_idempotency};
pub use validated_order::build_validated_order;
