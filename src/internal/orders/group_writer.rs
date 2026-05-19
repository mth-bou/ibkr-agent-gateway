//! Group order writer boundaries.

use super::{IdempotencyKey, LiveOrderWriter};
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

/// Group writer that delegates each bracket leg to the configured live writer.
///
/// This is a broker writer boundary, but it does not claim broker-native OCA
/// atomicity. Deployments that need native bracket/OCA semantics should wire a
/// dedicated broker adapter when one is available.
pub struct SequentialLiveOrderGroupWriter<'a> {
    writer: &'a dyn LiveOrderWriter,
}

impl<'a> SequentialLiveOrderGroupWriter<'a> {
    /// Creates a sequential group writer around a single-order live writer.
    #[must_use]
    pub const fn new(writer: &'a dyn LiveOrderWriter) -> Self {
        Self { writer }
    }
}

#[async_trait]
impl LiveOrderGroupWriter for SequentialLiveOrderGroupWriter<'_> {
    async fn submit_live_group(
        &self,
        group: &ValidatedOrderGroup,
        idempotency_key: &IdempotencyKey,
    ) -> Result<GroupSubmitReceipt, GatewayError> {
        let parent_key = leg_key(idempotency_key, "parent")?;
        let take_profit_key = leg_key(idempotency_key, "take-profit")?;
        let stop_loss_key = leg_key(idempotency_key, "stop-loss")?;
        let parent = self.writer.submit_live(&group.parent, &parent_key).await?;
        let take_profit = self
            .writer
            .submit_live(&group.take_profit, &take_profit_key)
            .await?;
        let stop_loss = self
            .writer
            .submit_live(&group.stop_loss, &stop_loss_key)
            .await?;

        Ok(GroupSubmitReceipt {
            broker_order_ids: vec![
                parent.broker_order_id,
                take_profit.broker_order_id,
                stop_loss.broker_order_id,
            ],
        })
    }
}

fn leg_key(idempotency_key: &IdempotencyKey, suffix: &str) -> Result<IdempotencyKey, GatewayError> {
    IdempotencyKey::new(format!("{}-{suffix}", idempotency_key.as_str()))
}

fn local_candidate_error() -> GatewayError {
    GatewayError::new(
        ErrorCode::OrderValidationFailed,
        "Local candidate group writer could not synthesize broker ids",
        false,
        Some("Use a non-empty idempotency key".to_string()),
    )
}
