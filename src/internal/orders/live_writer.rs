//! Live order broker writer boundary.
//!
//! [`submit_live_order`](super::submit_live_order) and
//! [`cancel_live_order`](super::cancel_live_order) delegate the broker-side
//! call to an implementation of [`LiveOrderWriter`]. This keeps risk gates,
//! idempotency, and audit responsibilities inside the gateway while letting
//! deployments wire any broker adapter — Client Portal Gateway, OAuth Web API,
//! a local candidate stub for offline smoke tests, or a refusing default.
//!
//! The trait returns broker-provided order ids, so live submit lifecycle
//! records carry a real broker-generated identifier rather than a synthetic
//! local placeholder.
//!
//! See [`crate::internal::cpapi::ClientPortalLiveWriter`] for the bundled
//! Client Portal Gateway implementation.

use super::idempotency::IdempotencyKey;
use crate::internal::domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError, ValidatedOrder};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Receipt returned by [`LiveOrderWriter::submit_live`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LiveSubmitReceipt {
    /// Broker-generated order identifier.
    pub broker_order_id: BrokerOrderId,
    /// Optional broker-reported status (e.g. `Submitted`, `PreSubmitted`).
    pub broker_status: Option<String>,
}

/// Receipt returned by [`LiveOrderWriter::cancel_live`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LiveCancelReceipt {
    /// Broker order identifier the cancel targeted.
    pub broker_order_id: BrokerOrderId,
    /// Whether the broker accepted the cancellation request.
    pub accepted: bool,
    /// Optional broker-reported status (e.g. `PendingCancel`, `Cancelled`).
    pub broker_status: Option<String>,
}

/// Broker live-order writer boundary.
#[async_trait]
pub trait LiveOrderWriter: Send + Sync {
    /// Submits a validated live order to the broker.
    async fn submit_live(
        &self,
        order: &ValidatedOrder,
        idempotency_key: &IdempotencyKey,
    ) -> Result<LiveSubmitReceipt, GatewayError>;

    /// Cancels a previously-submitted live order at the broker.
    async fn cancel_live(
        &self,
        account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        idempotency_key: &IdempotencyKey,
    ) -> Result<LiveCancelReceipt, GatewayError>;
}

/// Writer that returns deterministic local candidate ids derived from the
/// idempotency key. Use for CLI smoke tests and offline development.
///
/// This writer performs no network I/O and does not represent broker-side
/// execution. Operational deployments must wire a real broker adapter (for
/// example `ClientPortalLiveWriter` in the bundled `cpapi` module).
#[derive(Clone, Debug, Default)]
pub struct LocalCandidateLiveWriter;

#[async_trait]
impl LiveOrderWriter for LocalCandidateLiveWriter {
    async fn submit_live(
        &self,
        _order: &ValidatedOrder,
        idempotency_key: &IdempotencyKey,
    ) -> Result<LiveSubmitReceipt, GatewayError> {
        let raw = format!("local-candidate-{}", idempotency_key.as_str());
        let Some(broker_order_id) = BrokerOrderId::new(raw) else {
            return Err(local_candidate_error());
        };
        Ok(LiveSubmitReceipt {
            broker_order_id,
            broker_status: Some("LocalCandidate".to_string()),
        })
    }

    async fn cancel_live(
        &self,
        _account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<LiveCancelReceipt, GatewayError> {
        Ok(LiveCancelReceipt {
            broker_order_id: broker_order_id.clone(),
            accepted: true,
            broker_status: Some("Cancelled".to_string()),
        })
    }
}

/// Writer that refuses all live operations. Use as a fail-closed default when
/// no broker adapter has been wired.
#[derive(Clone, Debug, Default)]
pub struct RefusingLiveWriter;

#[async_trait]
impl LiveOrderWriter for RefusingLiveWriter {
    async fn submit_live(
        &self,
        _order: &ValidatedOrder,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<LiveSubmitReceipt, GatewayError> {
        Err(refusing_error("submit"))
    }

    async fn cancel_live(
        &self,
        _account_id: &AccountId,
        _broker_order_id: &BrokerOrderId,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<LiveCancelReceipt, GatewayError> {
        Err(refusing_error("cancel"))
    }
}

fn refusing_error(operation: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::LiveTradingDisabled,
        format!("No live order writer is wired for {operation}"),
        false,
        Some("Configure a LiveOrderWriter (e.g. ClientPortalLiveWriter)".to_string()),
    )
}

fn local_candidate_error() -> GatewayError {
    GatewayError::new(
        ErrorCode::OrderValidationFailed,
        "Local candidate live writer could not synthesize a broker order id",
        false,
        Some("Use a non-empty idempotency key".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::{LiveOrderWriter, LocalCandidateLiveWriter, RefusingLiveWriter};
    use crate::internal::domain::{
        AccountId, BrokerOrderId, ContractId, CurrencyCode, ErrorCode, Money, OrderIntentId,
        OrderSide, PreviewOrderType, Quantity, TimeInForce, ValidatedOrder, ValidatedOrderId,
    };
    use crate::internal::orders::IdempotencyKey;
    use rust_decimal::Decimal;
    use time::{Duration, OffsetDateTime};

    fn order() -> ValidatedOrder {
        let Some(currency) = CurrencyCode::new("USD") else {
            unreachable!("USD is a valid currency code");
        };
        ValidatedOrder {
            validated_order_id: ValidatedOrderId::new(),
            preview_id: crate::internal::domain::OrderPreviewId::new(),
            intent_id: OrderIntentId::new(),
            account_id: AccountId::from_static("U1234567"),
            contract_id: ContractId::from_static("265598"),
            symbol: Some("AAPL".to_string()),
            asset_class: Some(crate::internal::domain::AssetClass::Stock),
            side: OrderSide::Buy,
            quantity: Quantity::new(Decimal::ONE),
            order_type: PreviewOrderType::Limit,
            limit_price: Some(Money {
                amount: Decimal::new(100, 0),
                currency,
            }),
            time_in_force: TimeInForce::Day,
            expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
            warnings: Vec::new(),
        }
    }

    #[tokio::test]
    async fn local_candidate_submit_returns_deterministic_id() {
        let writer = LocalCandidateLiveWriter;
        let Ok(key) = IdempotencyKey::new("test-key") else {
            unreachable!("non-empty key");
        };
        let Ok(receipt) = writer.submit_live(&order(), &key).await else {
            unreachable!("local candidate writer must succeed");
        };
        assert_eq!(receipt.broker_order_id.as_str(), "local-candidate-test-key");
        assert_eq!(receipt.broker_status.as_deref(), Some("LocalCandidate"));
    }

    #[tokio::test]
    async fn local_candidate_cancel_echoes_broker_id() {
        let writer = LocalCandidateLiveWriter;
        let Ok(key) = IdempotencyKey::new("test-key") else {
            unreachable!("non-empty key");
        };
        let order_id = BrokerOrderId::from_static("known-broker-id");
        let Ok(receipt) = writer
            .cancel_live(&AccountId::from_static("U1234567"), &order_id, &key)
            .await
        else {
            unreachable!("local candidate writer must succeed");
        };
        assert_eq!(receipt.broker_order_id, order_id);
        assert!(receipt.accepted);
    }

    #[tokio::test]
    async fn refusing_writer_refuses_submit_and_cancel() {
        let writer = RefusingLiveWriter;
        let Ok(key) = IdempotencyKey::new("k") else {
            unreachable!("non-empty key");
        };
        let order_id = BrokerOrderId::from_static("any");
        let submit_error = writer.submit_live(&order(), &key).await;
        let Err(submit_error) = submit_error else {
            unreachable!("RefusingLiveWriter must refuse submit");
        };
        assert_eq!(submit_error.code, ErrorCode::LiveTradingDisabled);

        let cancel_error = writer
            .cancel_live(&AccountId::from_static("U1234567"), &order_id, &key)
            .await;
        let Err(cancel_error) = cancel_error else {
            unreachable!("RefusingLiveWriter must refuse cancel");
        };
        assert_eq!(cancel_error.code, ErrorCode::LiveTradingDisabled);
    }
}
