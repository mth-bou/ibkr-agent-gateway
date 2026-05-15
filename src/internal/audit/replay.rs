//! Replay fixture models for CI-safe audit regression checks.

use super::{AuditDecision, AuditEvent};
use ibkr_domain::{ErrorCode, GatewayError};
use serde::{Deserialize, Serialize};

/// Replay fixture for one redacted audit event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayCase {
    /// Stable case id.
    pub case_id: String,
    /// Redacted input event.
    pub input_event: AuditEvent,
    /// Expected decision.
    pub expected_decision: AuditDecision,
    /// Expected output shape.
    pub expected_output_shape: String,
    /// Expected secret scan result.
    pub secret_scan_expectation: SecretScanExpectation,
}

/// Expected secret scan result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretScanExpectation {
    /// No secret-like material should be present.
    Pass,
}

/// Replay result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayOutcome {
    /// Case id.
    pub case_id: String,
    /// Whether replay matched expectations.
    pub matched: bool,
    /// Replayed decision.
    pub decision: AuditDecision,
    /// Replayed output shape.
    pub output_shape: String,
}

/// Replays one redacted fixture without a live broker dependency.
pub fn replay_case(case: &ReplayCase) -> Result<ReplayOutcome, GatewayError> {
    assert_replay_secret_safe(case)?;
    let output_shape = output_shape_for_event(&case.input_event);
    let matched = case.input_event.decision == case.expected_decision
        && output_shape == case.expected_output_shape;

    Ok(ReplayOutcome {
        case_id: case.case_id.clone(),
        matched,
        decision: case.input_event.decision,
        output_shape,
    })
}

fn output_shape_for_event(event: &AuditEvent) -> String {
    match event.error_code {
        Some(error_code) => format!("refusal::{error_code:?}"),
        None => format!("decision::{:?}", event.decision).to_ascii_lowercase(),
    }
}

fn assert_replay_secret_safe(case: &ReplayCase) -> Result<(), GatewayError> {
    let rendered = serde_json::to_string(case).map_err(|_| {
        GatewayError::new(
            ErrorCode::OutputUnsafe,
            "Failed to serialize replay fixture",
            false,
            Some("Inspect replay fixture serialization".to_string()),
        )
    })?;
    let lowered = rendered.to_ascii_lowercase();

    for marker in [
        "bearer ",
        "client_secret",
        "cookie=",
        "refresh_token",
        "password=",
        "/home/",
        "\\users\\",
    ] {
        if lowered.contains(marker) {
            return Err(GatewayError::new(
                ErrorCode::OutputUnsafe,
                "Replay fixture contains secret-like material",
                false,
                Some("Use redacted audit fixtures only".to_string()),
            ));
        }
    }

    Ok(())
}
