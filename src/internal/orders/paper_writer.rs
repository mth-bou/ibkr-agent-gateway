//! Paper order broker writer boundary.
//!
//! Paper submit and cancel use the same broker-side order endpoints as live
//! trading, but against paper accounts (`DU*`). Keeping the writer pluggable
//! lets local tests stay offline while production-like paper validation can
//! exercise the real Client Portal Gateway path before live trading is enabled.

use super::idempotency::IdempotencyKey;
use crate::internal::domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError, ValidatedOrder};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Receipt returned by [`PaperOrderWriter::submit_paper`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaperSubmitReceipt {
    /// Broker-generated paper order identifier.
    pub broker_order_id: BrokerOrderId,
    /// Optional broker-reported status.
    pub broker_status: Option<String>,
}

/// Receipt returned by [`PaperOrderWriter::cancel_paper`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaperCancelReceipt {
    /// Broker paper order identifier the cancel targeted.
    pub broker_order_id: BrokerOrderId,
    /// Whether the broker accepted the cancellation request.
    pub accepted: bool,
    /// Optional broker-reported status.
    pub broker_status: Option<String>,
}

/// Broker paper-order writer boundary.
#[async_trait]
pub trait PaperOrderWriter: Send + Sync {
    /// Submits a validated paper order to the broker.
    async fn submit_paper(
        &self,
        order: &ValidatedOrder,
        idempotency_key: &IdempotencyKey,
    ) -> Result<PaperSubmitReceipt, GatewayError>;

    /// Cancels a paper order at the broker.
    async fn cancel_paper(
        &self,
        account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        idempotency_key: &IdempotencyKey,
    ) -> Result<PaperCancelReceipt, GatewayError>;
}

/// Offline writer that returns deterministic local candidate ids.
#[derive(Clone, Debug, Default)]
pub struct LocalCandidatePaperWriter;

#[async_trait]
impl PaperOrderWriter for LocalCandidatePaperWriter {
    async fn submit_paper(
        &self,
        _order: &ValidatedOrder,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<PaperSubmitReceipt, GatewayError> {
        Ok(PaperSubmitReceipt {
            broker_order_id: BrokerOrderId::from_static("paper-order-local"),
            broker_status: Some("LocalCandidate".to_string()),
        })
    }

    async fn cancel_paper(
        &self,
        _account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<PaperCancelReceipt, GatewayError> {
        Ok(PaperCancelReceipt {
            broker_order_id: broker_order_id.clone(),
            accepted: true,
            broker_status: Some("LocalCandidate".to_string()),
        })
    }
}

/// Writer that refuses all paper operations when no adapter is wired.
#[derive(Clone, Debug, Default)]
pub struct RefusingPaperWriter;

#[async_trait]
impl PaperOrderWriter for RefusingPaperWriter {
    async fn submit_paper(
        &self,
        _order: &ValidatedOrder,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<PaperSubmitReceipt, GatewayError> {
        Err(refusing_error("submit"))
    }

    async fn cancel_paper(
        &self,
        _account_id: &AccountId,
        _broker_order_id: &BrokerOrderId,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<PaperCancelReceipt, GatewayError> {
        Err(refusing_error("cancel"))
    }
}

fn refusing_error(operation: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperTradingDisabled,
        format!("No paper order writer is wired for {operation}"),
        false,
        Some("Configure a PaperOrderWriter (e.g. ClientPortalPaperWriter)".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::{LocalCandidatePaperWriter, PaperOrderWriter, RefusingPaperWriter};
    use crate::internal::domain::{AccountId, BrokerOrderId, ErrorCode};
    use crate::internal::orders::IdempotencyKey;

    #[tokio::test]
    async fn local_candidate_cancel_echoes_broker_id() {
        let writer = LocalCandidatePaperWriter;
        let Ok(key) = IdempotencyKey::new("paper-key") else {
            unreachable!("non-empty key");
        };
        let order_id = BrokerOrderId::from_static("paper-order-local");
        let Ok(receipt) = writer
            .cancel_paper(&AccountId::from_static("DU1234567"), &order_id, &key)
            .await
        else {
            unreachable!("local candidate writer must succeed");
        };
        assert_eq!(receipt.broker_order_id, order_id);
        assert!(receipt.accepted);
    }

    #[tokio::test]
    async fn refusing_writer_refuses_cancel() {
        let writer = RefusingPaperWriter;
        let Ok(key) = IdempotencyKey::new("paper-key") else {
            unreachable!("non-empty key");
        };
        let error = writer
            .cancel_paper(
                &AccountId::from_static("DU1234567"),
                &BrokerOrderId::from_static("any"),
                &key,
            )
            .await;
        let Err(error) = error else {
            unreachable!("RefusingPaperWriter must refuse cancel");
        };
        assert_eq!(error.code, ErrorCode::PaperTradingDisabled);
    }
}
