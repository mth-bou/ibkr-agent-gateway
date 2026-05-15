//! Paper-to-live migration checks.

use ibkr_domain::{ErrorCode, GatewayError, LocalUserId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Operator checklist required before live trading.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaperToLiveMigrationChecklist {
    /// Paper trading flow has been validated.
    pub paper_trading_validated: bool,
    /// Approval behavior has been reviewed.
    pub approvals_reviewed: bool,
    /// Live limits have been reviewed.
    pub limits_reviewed: bool,
    /// Kill switch has been tested.
    pub kill_switch_tested: bool,
    /// Incident runbook has been reviewed.
    pub incident_runbook_reviewed: bool,
    /// Operator that acknowledged the checklist.
    pub acknowledged_by: Option<LocalUserId>,
    /// Acknowledgement timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    pub acknowledged_at: Option<OffsetDateTime>,
}

impl PaperToLiveMigrationChecklist {
    /// Returns an acknowledged checklist for deterministic tests and CLI smoke flows.
    #[must_use]
    pub fn acknowledged(acknowledged_by: LocalUserId) -> Self {
        Self {
            paper_trading_validated: true,
            approvals_reviewed: true,
            limits_reviewed: true,
            kill_switch_tested: true,
            incident_runbook_reviewed: true,
            acknowledged_by: Some(acknowledged_by),
            acknowledged_at: Some(OffsetDateTime::now_utc()),
        }
    }

    /// Returns whether every migration check has been acknowledged.
    #[must_use]
    pub const fn is_acknowledged(&self) -> bool {
        self.paper_trading_validated
            && self.approvals_reviewed
            && self.limits_reviewed
            && self.kill_switch_tested
            && self.incident_runbook_reviewed
            && self.acknowledged_by.is_some()
            && self.acknowledged_at.is_some()
    }
}

/// Validates the paper-to-live migration checklist.
pub fn validate_paper_to_live_migration(
    checklist: &PaperToLiveMigrationChecklist,
) -> Result<(), GatewayError> {
    if checklist.is_acknowledged() {
        Ok(())
    } else {
        Err(GatewayError::new(
            ErrorCode::LiveMigrationRequired,
            "Paper-to-live checklist is not complete",
            false,
            Some("Complete the paper-to-live checklist before live trading".to_string()),
        ))
    }
}
