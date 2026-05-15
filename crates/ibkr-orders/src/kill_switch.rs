//! Live trading kill switch model and in-memory storage.

use ibkr_domain::{AuditEventId, LocalUserId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Live trading kill switch state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KillSwitchState {
    /// Live submit/cancel is allowed to proceed to other gates.
    Open,
    /// Live submit/cancel is refused immediately.
    Closed,
}

/// Current kill switch record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KillSwitch {
    /// Current state.
    pub state: KillSwitchState,
    /// Operator that last changed the state.
    pub changed_by: LocalUserId,
    /// Change timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub changed_at: OffsetDateTime,
    /// Safe operator reason.
    pub reason: String,
    /// Optional audit event id for the change.
    pub audit_event_id: Option<AuditEventId>,
}

impl KillSwitch {
    /// Creates a closed kill switch record.
    #[must_use]
    pub fn closed(changed_by: LocalUserId, reason: impl Into<String>) -> Self {
        Self {
            state: KillSwitchState::Closed,
            changed_by,
            changed_at: OffsetDateTime::now_utc(),
            reason: reason.into(),
            audit_event_id: None,
        }
    }

    /// Creates an open kill switch record.
    #[must_use]
    pub fn open(changed_by: LocalUserId, reason: impl Into<String>) -> Self {
        Self {
            state: KillSwitchState::Open,
            changed_by,
            changed_at: OffsetDateTime::now_utc(),
            reason: reason.into(),
            audit_event_id: None,
        }
    }

    /// Returns true when live trading may proceed to later gates.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        matches!(self.state, KillSwitchState::Open)
    }

    /// Adds audit correlation to this state change.
    #[must_use]
    pub fn with_audit_event_id(mut self, audit_event_id: AuditEventId) -> Self {
        self.audit_event_id = Some(audit_event_id);
        self
    }
}

/// In-memory kill switch storage for deterministic local tests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KillSwitchStore {
    current: KillSwitch,
}

impl KillSwitchStore {
    /// Creates storage initialized to closed.
    #[must_use]
    pub fn new_closed(changed_by: LocalUserId, reason: impl Into<String>) -> Self {
        Self {
            current: KillSwitch::closed(changed_by, reason),
        }
    }

    /// Returns the current record.
    #[must_use]
    pub const fn current(&self) -> &KillSwitch {
        &self.current
    }

    /// Replaces the current state.
    pub fn set(&mut self, next: KillSwitch) {
        self.current = next;
    }

    /// Opens the switch.
    pub fn open(&mut self, changed_by: LocalUserId, reason: impl Into<String>) {
        self.set(KillSwitch::open(changed_by, reason));
    }

    /// Closes the switch.
    pub fn close(&mut self, changed_by: LocalUserId, reason: impl Into<String>) {
        self.set(KillSwitch::closed(changed_by, reason));
    }

    /// Emergency-disables live trading immediately.
    pub fn emergency_disable(&mut self, changed_by: LocalUserId, reason: impl Into<String>) {
        self.close(changed_by, reason);
    }
}
