//! Audit retention configuration for live write workflows.

use crate::internal::domain::{ErrorCode, GatewayError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Audit retention config.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AuditRetentionConfig {
    /// Retention days for live write audit events.
    pub live_write_retention_days: u32,
    /// Whether an export is required before purge.
    pub export_required_before_purge: bool,
    /// Whether live write events are immutable.
    pub immutable_live_events: bool,
}

impl Default for AuditRetentionConfig {
    fn default() -> Self {
        Self {
            live_write_retention_days: 2_555,
            export_required_before_purge: true,
            immutable_live_events: true,
        }
    }
}

/// Validates retention policy for live trading.
pub fn validate_audit_retention_config(
    config: &AuditRetentionConfig,
    live_enabled: bool,
) -> Result<(), GatewayError> {
    if !live_enabled {
        return Ok(());
    }

    if config.live_write_retention_days < 2_555 {
        return Err(retention_error(
            "Live audit retention must be at least 2555 days",
            "Increase live_write_retention_days",
        ));
    }

    if !config.export_required_before_purge {
        return Err(retention_error(
            "Live audit retention requires export before purge",
            "Enable export_required_before_purge",
        ));
    }

    if !config.immutable_live_events {
        return Err(retention_error(
            "Live audit events must be immutable",
            "Enable immutable_live_events",
        ));
    }

    Ok(())
}

fn retention_error(message: &str, user_action: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::AuditWriteFailed,
        message,
        false,
        Some(user_action.to_string()),
    )
}
