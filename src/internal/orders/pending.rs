//! Shared pending idempotency cleanup after writer or gate errors.

use super::IdempotencyKey;
use crate::internal::audit::SqliteAuditWriter;
use crate::internal::domain::{ErrorCode, GatewayError};

/// Keeps writer-boundary failures for recovery and deletes pre-writer failures.
pub async fn handle_pending_order_error(
    audit_writer: &SqliteAuditWriter,
    idempotency_key: &IdempotencyKey,
    request_hash: &str,
    error: &GatewayError,
) -> Result<(), GatewayError> {
    if is_writer_boundary_error(error.code) {
        audit_writer
            .mark_order_failed_after_writer(idempotency_key, request_hash, error)
            .await
    } else {
        audit_writer
            .delete_order_pending(idempotency_key, request_hash)
            .await
    }
}

const fn is_writer_boundary_error(code: ErrorCode) -> bool {
    matches!(
        code,
        ErrorCode::BrokerBackendUnavailable
            | ErrorCode::BrokerResponseInvalid
            | ErrorCode::BrokerSessionRequired
            | ErrorCode::OrderValidationFailed
    )
}
