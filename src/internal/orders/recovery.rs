//! Crash recovery for order idempotency write-ahead records.

use super::lifecycle::{
    LiveOrderLifecycleRecord, LiveOrderLifecycleStatus, PaperOrderLifecycleRecord,
    PaperOrderLifecycleStatus,
};
use crate::internal::audit::{
    OrderIdempotencyOperation, OrderIdempotencyRecoveryContext, OrderIdempotencyWorkflow,
    SqliteAuditWriter,
};
use crate::internal::backend::IbkrBackend;
use crate::internal::domain::{ErrorCode, GatewayError, ReadOnlyOrderRecord, ReadOnlyOrderStatus};
use crate::internal::orders::IdempotencyKey;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::warn;

/// Summary returned after scanning pending writer records.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct OrderIdempotencyRecoveryReport {
    /// Pending records inspected.
    pub scanned: usize,
    /// Records completed from broker-side order state.
    pub completed: usize,
    /// Records cleared because broker state says no order exists.
    pub cleared_missing: usize,
    /// Records left pending because they lack recovery context or backend access.
    pub unresolved: usize,
}

/// Scans `pending_writer` idempotency records and resolves broker-side state.
///
/// Submit records are looked up by their idempotency key, which is sent to IBKR
/// as `cOID`. Cancel records are looked up by the known broker order id.
pub async fn recover_pending_order_idempotency(
    audit_writer: &SqliteAuditWriter,
    backend: &dyn IbkrBackend,
) -> Result<OrderIdempotencyRecoveryReport, GatewayError> {
    let pending = audit_writer.pending_order_idempotency_records().await?;
    let mut report = OrderIdempotencyRecoveryReport {
        scanned: pending.len(),
        ..OrderIdempotencyRecoveryReport::default()
    };

    for record in pending {
        let Some(context) = record.recovery_context else {
            report.unresolved += 1;
            continue;
        };
        let idempotency_key = match IdempotencyKey::new(record.idempotency_key.clone()) {
            Ok(idempotency_key) => idempotency_key,
            Err(_) => {
                report.unresolved += 1;
                continue;
            }
        };
        let lookup_id = lookup_id(&context, &idempotency_key);
        let order = match backend
            .order_status(&context.account_id, lookup_id.as_str())
            .await
        {
            Ok(order) => order,
            Err(error) if error.code == ErrorCode::BrokerCapabilityUnavailable => {
                audit_writer
                    .delete_order_pending(&idempotency_key, &record.request_hash)
                    .await?;
                report.cleared_missing += 1;
                continue;
            }
            Err(error) => {
                warn!(
                    target: "orders.recovery",
                    error_code = ?error.code,
                    idempotency_key = record.idempotency_key.as_str(),
                    "order idempotency recovery left pending record unresolved"
                );
                report.unresolved += 1;
                continue;
            }
        };
        let payload = recovered_payload(&context, order)?;
        audit_writer
            .complete_order_idempotency(&idempotency_key, &record.request_hash, &payload)
            .await?;
        report.completed += 1;
    }

    Ok(report)
}

fn lookup_id(
    context: &OrderIdempotencyRecoveryContext,
    idempotency_key: &IdempotencyKey,
) -> String {
    match context.operation {
        OrderIdempotencyOperation::Submit => idempotency_key.as_str().to_string(),
        OrderIdempotencyOperation::Cancel => context
            .broker_order_id
            .as_ref()
            .map(|broker_order_id| broker_order_id.as_str().to_string())
            .unwrap_or_else(|| idempotency_key.as_str().to_string()),
    }
}

fn recovered_payload(
    context: &OrderIdempotencyRecoveryContext,
    order: ReadOnlyOrderRecord,
) -> Result<serde_json::Value, GatewayError> {
    let updated_at = order.updated_at.unwrap_or_else(OffsetDateTime::now_utc);
    match context.workflow {
        OrderIdempotencyWorkflow::Paper => {
            let status = paper_status(context.operation, order.status);
            serde_json::to_value(PaperOrderLifecycleRecord {
                account_id: order.account_id,
                broker_order_id: order.broker_order_id,
                status,
                updated_at,
            })
        }
        OrderIdempotencyWorkflow::Live => {
            let status = live_status(context.operation, order.status);
            serde_json::to_value(LiveOrderLifecycleRecord {
                account_id: order.account_id,
                broker_order_id: order.broker_order_id,
                status,
                execution_correlation: None,
                updated_at,
            })
        }
    }
    .map_err(|_| {
        GatewayError::new(
            ErrorCode::AuditWriteFailed,
            "Unable to serialize recovered order lifecycle",
            true,
            Some("Inspect pending idempotency recovery".to_string()),
        )
    })
}

const fn paper_status(
    operation: OrderIdempotencyOperation,
    status: ReadOnlyOrderStatus,
) -> PaperOrderLifecycleStatus {
    match status {
        ReadOnlyOrderStatus::Open => PaperOrderLifecycleStatus::Open,
        ReadOnlyOrderStatus::Filled => PaperOrderLifecycleStatus::Filled,
        ReadOnlyOrderStatus::Cancelled => PaperOrderLifecycleStatus::Cancelled,
        ReadOnlyOrderStatus::Unknown => match operation {
            OrderIdempotencyOperation::Submit => PaperOrderLifecycleStatus::Submitted,
            OrderIdempotencyOperation::Cancel => PaperOrderLifecycleStatus::Cancelled,
        },
    }
}

const fn live_status(
    operation: OrderIdempotencyOperation,
    status: ReadOnlyOrderStatus,
) -> LiveOrderLifecycleStatus {
    match status {
        ReadOnlyOrderStatus::Open => LiveOrderLifecycleStatus::Open,
        ReadOnlyOrderStatus::Filled => LiveOrderLifecycleStatus::Filled,
        ReadOnlyOrderStatus::Cancelled => LiveOrderLifecycleStatus::Cancelled,
        ReadOnlyOrderStatus::Unknown => match operation {
            OrderIdempotencyOperation::Submit => LiveOrderLifecycleStatus::Submitted,
            OrderIdempotencyOperation::Cancel => LiveOrderLifecycleStatus::Cancelled,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::recover_pending_order_idempotency;
    use crate::internal::audit::{
        AuditHmacKey, OrderIdempotencyOperation, OrderIdempotencyRecoveryContext,
        OrderIdempotencyWorkflow, SqliteAuditWriter,
    };
    use crate::internal::backend::{BackendResult, IbkrBackend};
    use crate::internal::domain::{
        AccountId, BrokerAccount, BrokerOrderId, BrokerSessionStatus, ContractCandidate,
        ContractId, ErrorCode, GatewayError, HistoricalBar, HistoricalBarsRequest, MarketSnapshot,
        ReadOnlyOrderRecord, ReadOnlyOrderStatus,
    };
    use crate::internal::orders::IdempotencyKey;
    use async_trait::async_trait;
    use std::sync::Arc;

    #[tokio::test]
    async fn recovery_completes_submit_pending_from_client_order_id()
    -> Result<(), Box<dyn std::error::Error>> {
        let writer =
            SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?))
                .await?;
        let idempotency_key = IdempotencyKey::new("coid-recovery-key")?;
        let request_hash = "submit-request-hash";
        let context = OrderIdempotencyRecoveryContext {
            workflow: OrderIdempotencyWorkflow::Live,
            operation: OrderIdempotencyOperation::Submit,
            account_id: AccountId::from_static("U1234567"),
            broker_order_id: None,
        };
        writer
            .insert_order_pending_with_context(&idempotency_key, request_hash, Some(&context))
            .await?;

        let backend = RecoveryBackend::found(
            "coid-recovery-key",
            ReadOnlyOrderRecord {
                account_id: AccountId::from_static("U1234567"),
                broker_order_id: BrokerOrderId::from_static("broker-123"),
                client_order_id: Some("coid-recovery-key".to_string()),
                status: ReadOnlyOrderStatus::Open,
                side: None,
                quantity: None,
                filled_quantity: None,
                contract_id: None,
                limit_price: None,
                currency: None,
                created_at: None,
                updated_at: None,
            },
        );

        let report = recover_pending_order_idempotency(&writer, &backend).await?;
        assert_eq!(report.scanned, 1);
        assert_eq!(report.completed, 1);
        assert!(writer.pending_order_idempotency_records().await?.is_empty());
        let replayed = writer
            .replay_order_idempotency(&idempotency_key, request_hash)
            .await?
            .ok_or("recovered idempotency record must replay")?;
        assert_eq!(replayed["broker_order_id"], "broker-123");
        assert_eq!(replayed["status"], "open");
        Ok(())
    }

    #[tokio::test]
    async fn recovery_clears_pending_when_broker_reports_missing()
    -> Result<(), Box<dyn std::error::Error>> {
        let writer =
            SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?))
                .await?;
        let idempotency_key = IdempotencyKey::new("missing-coid-key")?;
        let request_hash = "missing-request-hash";
        let context = OrderIdempotencyRecoveryContext {
            workflow: OrderIdempotencyWorkflow::Paper,
            operation: OrderIdempotencyOperation::Submit,
            account_id: AccountId::from_static("DU1234567"),
            broker_order_id: None,
        };
        writer
            .insert_order_pending_with_context(&idempotency_key, request_hash, Some(&context))
            .await?;

        let report =
            recover_pending_order_idempotency(&writer, &RecoveryBackend::missing()).await?;
        assert_eq!(report.scanned, 1);
        assert_eq!(report.cleared_missing, 1);
        assert!(writer.pending_order_idempotency_records().await?.is_empty());
        assert!(
            writer
                .replay_order_idempotency(&idempotency_key, request_hash)
                .await?
                .is_none()
        );
        Ok(())
    }

    struct RecoveryBackend {
        expected_lookup: Option<String>,
        order: Option<ReadOnlyOrderRecord>,
    }

    impl RecoveryBackend {
        fn found(expected_lookup: &str, order: ReadOnlyOrderRecord) -> Self {
            Self {
                expected_lookup: Some(expected_lookup.to_string()),
                order: Some(order),
            }
        }

        const fn missing() -> Self {
            Self {
                expected_lookup: None,
                order: None,
            }
        }
    }

    #[async_trait]
    impl IbkrBackend for RecoveryBackend {
        async fn session_status(&self) -> BackendResult<BrokerSessionStatus> {
            Err(unused_backend_method())
        }

        async fn keepalive(&self) -> BackendResult<BrokerSessionStatus> {
            Err(unused_backend_method())
        }

        async fn list_accounts(&self) -> BackendResult<Vec<BrokerAccount>> {
            Err(unused_backend_method())
        }

        async fn account_summary(
            &self,
            _account_id: &AccountId,
        ) -> BackendResult<serde_json::Value> {
            Err(unused_backend_method())
        }

        async fn portfolio_snapshot(
            &self,
            _account_id: &AccountId,
        ) -> BackendResult<serde_json::Value> {
            Err(unused_backend_method())
        }

        async fn positions(
            &self,
            _account_id: &AccountId,
        ) -> BackendResult<Vec<serde_json::Value>> {
            Err(unused_backend_method())
        }

        async fn search_contracts(&self, _query: &str) -> BackendResult<Vec<ContractCandidate>> {
            Err(unused_backend_method())
        }

        async fn resolve_contract(&self, _query: &str) -> BackendResult<ContractCandidate> {
            Err(unused_backend_method())
        }

        async fn market_snapshot(
            &self,
            _contract_id: &ContractId,
        ) -> BackendResult<MarketSnapshot> {
            Err(unused_backend_method())
        }

        async fn historical_bars(
            &self,
            _request: &HistoricalBarsRequest,
        ) -> BackendResult<Vec<HistoricalBar>> {
            Err(unused_backend_method())
        }

        async fn orders(&self, _account_id: &AccountId) -> BackendResult<Vec<ReadOnlyOrderRecord>> {
            Err(unused_backend_method())
        }

        async fn order_status(
            &self,
            _account_id: &AccountId,
            broker_order_id: &str,
        ) -> BackendResult<ReadOnlyOrderRecord> {
            if self.expected_lookup.as_deref() == Some(broker_order_id)
                && let Some(order) = &self.order
            {
                return Ok(order.clone());
            }
            Err(GatewayError::new(
                ErrorCode::BrokerCapabilityUnavailable,
                "Order status was not found",
                false,
                Some("Use a known broker order id".to_string()),
            ))
        }

        async fn executions(
            &self,
            _account_id: &AccountId,
        ) -> BackendResult<Vec<serde_json::Value>> {
            Err(unused_backend_method())
        }
    }

    fn unused_backend_method() -> GatewayError {
        GatewayError::new(
            ErrorCode::BrokerCapabilityUnavailable,
            "Unused backend method",
            false,
            None,
        )
    }
}
