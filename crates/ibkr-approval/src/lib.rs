//! Approval records for paper order workflows.

pub mod model;
pub mod service;

pub use model::{ApprovalId, ApprovalRecord, ApprovalStatus};
pub use service::ApprovalService;
