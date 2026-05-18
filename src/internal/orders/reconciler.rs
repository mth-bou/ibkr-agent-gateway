//! Live order lifecycle reconciliation.

use super::{LiveOrderLifecycleRecord, LiveOrderLifecycleStatus};
use crate::internal::audit::{
    AuditDecision, AuditEvent, AuditEventType, AuditResultStatus, SqliteAuditWriter,
};
use crate::internal::backend::IbkrBackend;
use crate::internal::domain::{
    AuditEventId, GatewayError, LocalUserId, ReadOnlyOrderRecord, RequestId, SessionId,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use time::OffsetDateTime;
use tracing::warn;

/// Summary returned after one reconciliation pass.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LiveOrderReconciliationReport {
    /// Pending live orders inspected.
    pub scanned: usize,
    /// Orders whose broker status changed since the last recorded status.
    pub transitions: usize,
    /// Orders removed from the backlog after reaching a terminal state.
    pub completed: usize,
    /// Orders still non-terminal with no status transition.
    pub unchanged: usize,
    /// Orders that could not be reconciled during this pass.
    pub unresolved: usize,
}

/// Polls broker status once for every pending live order.
///
/// The continuous runtime can call this function on its configured interval.
/// It keeps storage updates idempotent: non-terminal statuses stay in the
/// backlog with a fresh poll timestamp, terminal statuses are removed.
pub async fn reconcile_live_orders_once(
    audit_writer: &SqliteAuditWriter,
    backend: &dyn IbkrBackend,
) -> Result<LiveOrderReconciliationReport, GatewayError> {
    let pending = audit_writer.pending_live_orders().await?;
    let mut report = LiveOrderReconciliationReport {
        scanned: pending.len(),
        ..LiveOrderReconciliationReport::default()
    };

    for pending_order in pending {
        let broker_order = match backend
            .order_status(
                &pending_order.account_id,
                pending_order.broker_order_id.as_str(),
            )
            .await
        {
            Ok(order) => order,
            Err(error) => {
                warn!(
                    target: "orders.reconciler",
                    error_code = ?error.code,
                    broker_order_id = %pending_order.broker_order_id.as_str(),
                    "live order reconciliation left order unresolved"
                );
                report.unresolved += 1;
                continue;
            }
        };

        let lifecycle = lifecycle_from_broker_order(broker_order);
        if lifecycle.status != pending_order.last_status {
            audit_writer
                .append(&build_live_lifecycle_event(
                    &lifecycle,
                    pending_order.last_status,
                ))
                .await?;
            report.transitions += 1;
        } else {
            report.unchanged += 1;
        }

        if lifecycle.status.is_terminal() {
            audit_writer
                .remove_live_order_pending(&lifecycle.account_id, &lifecycle.broker_order_id)
                .await?;
            report.completed += 1;
        } else {
            audit_writer.upsert_live_order_pending(&lifecycle).await?;
        }
    }

    Ok(report)
}

fn lifecycle_from_broker_order(order: ReadOnlyOrderRecord) -> LiveOrderLifecycleRecord {
    LiveOrderLifecycleRecord {
        account_id: order.account_id,
        broker_order_id: order.broker_order_id,
        status: LiveOrderLifecycleStatus::from_read_only_order_status(order.status),
        execution_correlation: None,
        updated_at: order.updated_at.unwrap_or_else(OffsetDateTime::now_utc),
    }
}

fn build_live_lifecycle_event(
    lifecycle: &LiveOrderLifecycleRecord,
    previous_status: LiveOrderLifecycleStatus,
) -> AuditEvent {
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "broker_order_id".to_string(),
        json!(lifecycle.broker_order_id.as_str()),
    );
    metadata.insert("previous_status".to_string(), json!(previous_status));
    metadata.insert("current_status".to_string(), json!(lifecycle.status));

    AuditEvent {
        event_id: AuditEventId::new(),
        event_type: AuditEventType::LiveOrderLifecycleChanged,
        timestamp: OffsetDateTime::now_utc(),
        user_id: LocalUserId::from_static("reconciler"),
        session_id: SessionId::new(),
        request_id: RequestId::new(),
        account_id_hash: None,
        tool_name: Some("ibkr_live_order_reconciler".to_string()),
        scopes: Vec::new(),
        decision: AuditDecision::Allow,
        result_status: AuditResultStatus::Completed,
        error_code: None,
        input_hash: None,
        output_hash: None,
        redactions: Vec::new(),
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::reconcile_live_orders_once;
    use crate::internal::audit::{
        AuditEventType, AuditHmacKey, AuditTailRequest, SqliteAuditWriter,
    };
    use crate::internal::backend::{BackendResult, IbkrBackend};
    use crate::internal::domain::{
        AccountId, BrokerAccount, BrokerOrderId, BrokerSessionStatus, ContractCandidate,
        ContractId, GatewayError, HistoricalBar, HistoricalBarsRequest, MarketSnapshot,
        ReadOnlyOrderRecord, ReadOnlyOrderStatus,
    };
    use crate::internal::orders::{LiveOrderLifecycleRecord, LiveOrderLifecycleStatus};
    use async_trait::async_trait;
    use std::sync::Arc;
    use time::OffsetDateTime;

    #[tokio::test]
    async fn reconciler_records_transition_and_removes_terminal_order()
    -> Result<(), Box<dyn std::error::Error>> {
        let writer =
            SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?))
                .await?;
        let lifecycle = LiveOrderLifecycleRecord {
            account_id: AccountId::from_static("DU1234567"),
            broker_order_id: BrokerOrderId::from_static("broker-1"),
            status: LiveOrderLifecycleStatus::Submitted,
            execution_correlation: None,
            updated_at: OffsetDateTime::now_utc(),
        };
        writer.upsert_live_order_pending(&lifecycle).await?;

        let backend = StaticOrderBackend::new(ReadOnlyOrderStatus::Filled);
        let report = reconcile_live_orders_once(&writer, &backend).await?;

        assert_eq!(report.scanned, 1);
        assert_eq!(report.transitions, 1);
        assert_eq!(report.completed, 1);
        assert!(writer.pending_live_orders().await?.is_empty());
        let tail = writer.tail(AuditTailRequest::new(1)).await?;
        assert_eq!(
            tail.events[0].event.event_type,
            AuditEventType::LiveOrderLifecycleChanged
        );
        assert_eq!(tail.events[0].event.metadata["current_status"], "filled");
        Ok(())
    }

    #[tokio::test]
    async fn reconciler_keeps_unchanged_non_terminal_order()
    -> Result<(), Box<dyn std::error::Error>> {
        let writer =
            SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?))
                .await?;
        let lifecycle = LiveOrderLifecycleRecord {
            account_id: AccountId::from_static("DU1234567"),
            broker_order_id: BrokerOrderId::from_static("broker-1"),
            status: LiveOrderLifecycleStatus::Open,
            execution_correlation: None,
            updated_at: OffsetDateTime::now_utc(),
        };
        writer.upsert_live_order_pending(&lifecycle).await?;

        let backend = StaticOrderBackend::new(ReadOnlyOrderStatus::Open);
        let report = reconcile_live_orders_once(&writer, &backend).await?;

        assert_eq!(report.scanned, 1);
        assert_eq!(report.unchanged, 1);
        assert_eq!(writer.pending_live_orders().await?.len(), 1);
        Ok(())
    }

    struct StaticOrderBackend {
        status: ReadOnlyOrderStatus,
    }

    impl StaticOrderBackend {
        fn new(status: ReadOnlyOrderStatus) -> Self {
            Self { status }
        }
    }

    #[async_trait]
    impl IbkrBackend for StaticOrderBackend {
        async fn session_status(&self) -> BackendResult<BrokerSessionStatus> {
            unimplemented!()
        }

        async fn keepalive(&self) -> BackendResult<BrokerSessionStatus> {
            unimplemented!()
        }

        async fn list_accounts(&self) -> BackendResult<Vec<BrokerAccount>> {
            unimplemented!()
        }

        async fn account_summary(
            &self,
            _account_id: &AccountId,
        ) -> BackendResult<serde_json::Value> {
            unimplemented!()
        }

        async fn portfolio_snapshot(
            &self,
            _account_id: &AccountId,
        ) -> BackendResult<serde_json::Value> {
            unimplemented!()
        }

        async fn positions(
            &self,
            _account_id: &AccountId,
        ) -> BackendResult<Vec<serde_json::Value>> {
            unimplemented!()
        }

        async fn search_contracts(&self, _query: &str) -> BackendResult<Vec<ContractCandidate>> {
            unimplemented!()
        }

        async fn resolve_contract(&self, _query: &str) -> BackendResult<ContractCandidate> {
            unimplemented!()
        }

        async fn market_snapshot(
            &self,
            _contract_id: &ContractId,
        ) -> BackendResult<MarketSnapshot> {
            unimplemented!()
        }

        async fn historical_bars(
            &self,
            _request: &HistoricalBarsRequest,
        ) -> BackendResult<Vec<HistoricalBar>> {
            unimplemented!()
        }

        async fn orders(&self, _account_id: &AccountId) -> BackendResult<Vec<ReadOnlyOrderRecord>> {
            unimplemented!()
        }

        async fn order_status(
            &self,
            account_id: &AccountId,
            order_lookup_id: &str,
        ) -> BackendResult<ReadOnlyOrderRecord> {
            Ok(ReadOnlyOrderRecord {
                account_id: account_id.clone(),
                broker_order_id: BrokerOrderId::new(order_lookup_id).ok_or_else(|| {
                    GatewayError::new(
                        crate::internal::domain::ErrorCode::OrderValidationFailed,
                        "Order lookup id is required",
                        false,
                        Some("Provide a broker or client order id".to_string()),
                    )
                })?,
                client_order_id: None,
                status: self.status,
                side: None,
                quantity: None,
                filled_quantity: None,
                contract_id: None,
                limit_price: None,
                currency: None,
                created_at: None,
                updated_at: Some(OffsetDateTime::now_utc()),
            })
        }

        async fn executions(
            &self,
            _account_id: &AccountId,
        ) -> BackendResult<Vec<serde_json::Value>> {
            unimplemented!()
        }
    }
}
