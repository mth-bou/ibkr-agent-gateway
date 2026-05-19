#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    audit::{AuditHmacKey, SqliteAuditWriter},
    auth::{ORDERS_PREVIEW, ScopeSet},
    backend::{FakeBackend, FakeFixtureStore},
    domain::ErrorCode,
    mcp::order_workflows::{McpOrderWorkflowContext, handle_order_preview},
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn mcp_preview_accepts_stop_stop_limit_and_trailing_stop()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let context = McpOrderWorkflowContext {
        backend: &backend,
        audit_writer: &writer,
    };
    let scopes = ScopeSet::local_with_preview([ORDERS_PREVIEW])?;

    let stop = handle_order_preview(
        &context,
        &scopes,
        &json!({
            "account_id": "DU1234567",
            "symbol": "AAPL",
            "side": "sell",
            "quantity": "1",
            "order_type": "stop",
            "stop_price": "95",
            "time_in_force": "day"
        }),
    )
    .await?;
    let stop_id = stop["preview_id"]
        .as_str()
        .ok_or("preview id should be serialized")?;
    let stop_record = writer
        .load_order_preview(&ibkr_agent_gateway::testing::domain::OrderPreviewId::parse(
            stop_id,
        )?)
        .await?
        .ok_or("preview record should be stored")?;
    assert_eq!(
        stop_record.validated_order.order_type,
        ibkr_agent_gateway::testing::domain::PreviewOrderType::Stop
    );
    assert!(stop_record.validated_order.stop_price.is_some());

    let stop_limit = handle_order_preview(
        &context,
        &scopes,
        &json!({
            "account_id": "DU1234567",
            "symbol": "AAPL",
            "side": "sell",
            "quantity": "1",
            "order_type": "stop_limit",
            "stop_price": "95",
            "limit_price": "94.50",
            "time_in_force": "day"
        }),
    )
    .await?;
    assert!(stop_limit["preview_id"].is_string());

    let trailing = handle_order_preview(
        &context,
        &scopes,
        &json!({
            "account_id": "DU1234567",
            "symbol": "AAPL",
            "side": "sell",
            "quantity": "1",
            "order_type": "trailing_stop",
            "trailing_amount": "2.5",
            "time_in_force": "day"
        }),
    )
    .await?;
    assert!(trailing["preview_id"].is_string());

    Ok(())
}

#[tokio::test]
async fn mcp_preview_refuses_market_and_missing_stop_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let context = McpOrderWorkflowContext {
        backend: &backend,
        audit_writer: &writer,
    };
    let scopes = ScopeSet::local_with_preview([ORDERS_PREVIEW])?;

    let market = handle_order_preview(
        &context,
        &scopes,
        &json!({
            "account_id": "DU1234567",
            "symbol": "AAPL",
            "side": "buy",
            "quantity": "1",
            "order_type": "market",
            "time_in_force": "day"
        }),
    )
    .await
    .err()
    .ok_or("market preview should be refused")?;
    assert_eq!(market.code, ErrorCode::OrderPolicyRefused);

    let missing_stop = handle_order_preview(
        &context,
        &scopes,
        &json!({
            "account_id": "DU1234567",
            "symbol": "AAPL",
            "side": "sell",
            "quantity": "1",
            "order_type": "stop",
            "time_in_force": "day"
        }),
    )
    .await
    .err()
    .ok_or("stop preview without stop price should be refused")?;
    assert_eq!(missing_stop.code, ErrorCode::OrderValidationFailed);

    Ok(())
}
