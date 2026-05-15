//! Audit query DTOs.

use super::event::AuditEvent;
use serde::{Deserialize, Serialize};

/// Request for recent audit events.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditTailRequest {
    /// Maximum number of events to return.
    pub limit: u32,
}

impl AuditTailRequest {
    /// Creates a bounded tail request.
    #[must_use]
    pub const fn new(limit: u32) -> Self {
        Self { limit }
    }

    /// Returns a normalized limit.
    #[must_use]
    pub const fn normalized_limit(self) -> u32 {
        if self.limit == 0 {
            100
        } else if self.limit > 500 {
            500
        } else {
            self.limit
        }
    }
}

/// One persisted audit event with local sequence metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditTailRecord {
    /// Monotonic SQLite sequence id.
    pub sequence_id: i64,
    /// Redacted audit event payload.
    pub event: AuditEvent,
}

/// Recent audit events.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditTail {
    /// Events ordered newest first.
    pub events: Vec<AuditTailRecord>,
}
