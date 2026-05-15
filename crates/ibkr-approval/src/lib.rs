//! Compatibility crate for approval helpers while root migration is in progress.

#[path = "../../../src/internal/approval/model.rs"]
pub mod model;
#[path = "../../../src/internal/approval/service.rs"]
pub mod service;

pub use model::{ApprovalId, ApprovalRecord, ApprovalStatus};
pub use service::ApprovalService;
