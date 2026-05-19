//! Group order writer boundaries.

use super::IdempotencyKey;
use crate::internal::domain::{BrokerOrderId, ErrorCode, GatewayError, ValidatedOrderGroup};
use async_trait::async_trait;

/// Receipt returned by a grouped order writer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupSubmitReceipt {
    /// Broker order ids, one per bracket leg.
    pub broker_order_ids: Vec<BrokerOrderId>,
}

/// Paper group writer boundary.
#[async_trait]
pub trait PaperOrderGroupWriter: Send + Sync {
    /// Submits a validated paper order group.
    async fn submit_paper_group(
        &self,
        group: &ValidatedOrderGroup,
        idempotency_key: &IdempotencyKey,
    ) -> Result<GroupSubmitReceipt, GatewayError>;
}

/// Live group writer boundary.
#[async_trait]
pub trait LiveOrderGroupWriter: Send + Sync {
    /// Submits a validated live order group.
    async fn submit_live_group(
        &self,
        group: &ValidatedOrderGroup,
        idempotency_key: &IdempotencyKey,
    ) -> Result<GroupSubmitReceipt, GatewayError>;
}

/// Offline paper writer for grouped smoke tests.
#[derive(Clone, Debug, Default)]
pub struct LocalCandidatePaperGroupWriter;

#[async_trait]
impl PaperOrderGroupWriter for LocalCandidatePaperGroupWriter {
    async fn submit_paper_group(
        &self,
        _group: &ValidatedOrderGroup,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<GroupSubmitReceipt, GatewayError> {
        Ok(GroupSubmitReceipt {
            broker_order_ids: vec![
                BrokerOrderId::from_static("paper-bracket-parent-local"),
                BrokerOrderId::from_static("paper-bracket-take-profit-local"),
                BrokerOrderId::from_static("paper-bracket-stop-loss-local"),
            ],
        })
    }
}

/// Offline live writer for grouped smoke tests.
#[derive(Clone, Debug, Default)]
pub struct LocalCandidateLiveGroupWriter;

#[async_trait]
impl LiveOrderGroupWriter for LocalCandidateLiveGroupWriter {
    async fn submit_live_group(
        &self,
        _group: &ValidatedOrderGroup,
        idempotency_key: &IdempotencyKey,
    ) -> Result<GroupSubmitReceipt, GatewayError> {
        Ok(GroupSubmitReceipt {
            broker_order_ids: vec![
                BrokerOrderId::new(format!("live-bracket-{}-parent", idempotency_key.as_str()))
                    .ok_or_else(local_candidate_error)?,
                BrokerOrderId::new(format!(
                    "live-bracket-{}-take-profit",
                    idempotency_key.as_str()
                ))
                .ok_or_else(local_candidate_error)?,
                BrokerOrderId::new(format!(
                    "live-bracket-{}-stop-loss",
                    idempotency_key.as_str()
                ))
                .ok_or_else(local_candidate_error)?,
            ],
        })
    }
}

fn local_candidate_error() -> GatewayError {
    GatewayError::new(
        ErrorCode::OrderValidationFailed,
        "Local candidate group writer could not synthesize broker ids",
        false,
        Some("Use a non-empty idempotency key".to_string()),
    )
}
