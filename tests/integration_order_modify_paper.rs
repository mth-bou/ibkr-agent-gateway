#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    audit::{AuditHmacKey, SqliteAuditWriter},
    auth::{ORDERS_PAPER_MODIFY, ScopeSet},
    backend::{FakeBackend, FakeFixtureStore},
    domain::ErrorCode,
    mcp::order_workflows::{McpOrderWorkflowContext, handle_paper_modify},
    orders::PaperOrderLifecycleStatus,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn mcp_paper_modify_records_and_replays_idempotency() -> Result<(), Box<dyn std::error::Error>>
{
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let context = McpOrderWorkflowContext {
        backend: &backend,
        audit_writer: &writer,
    };
    let scopes = ScopeSet::local_with_paper([ORDERS_PAPER_MODIFY])?;
    let args = json!({
        "account_id": "DU1234567",
        "broker_order_id": "paper-order-1",
        "idempotency_key": "paper-modify-1",
        "limit_price": "124.50"
    });

    let first = handle_paper_modify(&context, &scopes, &args).await?;
    let replay = handle_paper_modify(&context, &scopes, &args).await?;

    assert_eq!(first, replay);
    assert_eq!(first["account_id"], "DU1234567");
    assert_eq!(first["broker_order_id"], "paper-order-1");
    assert_eq!(
        serde_json::from_value::<PaperOrderLifecycleStatus>(first["status"].clone())?,
        PaperOrderLifecycleStatus::Open
    );
    Ok(())
}

#[tokio::test]
async fn mcp_paper_modify_refuses_empty_change_set() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let context = McpOrderWorkflowContext {
        backend: &backend,
        audit_writer: &writer,
    };
    let scopes = ScopeSet::local_with_paper([ORDERS_PAPER_MODIFY])?;

    let error = handle_paper_modify(
        &context,
        &scopes,
        &json!({
            "account_id": "DU1234567",
            "broker_order_id": "paper-order-1",
            "idempotency_key": "paper-modify-empty"
        }),
    )
    .await
    .err()
    .ok_or("empty modify should fail")?;

    assert_eq!(error.code, ErrorCode::OrderValidationFailed);
    Ok(())
}
