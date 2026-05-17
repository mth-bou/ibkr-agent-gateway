#![cfg(feature = "unstable-internal-test-support")]

#[path = "common/live.rs"]
mod live;

use ibkr_agent_gateway::testing::{
    approval::ApprovalService,
    audit::{AuditHmacKey, SqliteAuditWriter},
    auth::{ORDERS_LIVE_CANCEL, ORDERS_LIVE_SUBMIT, ScopeSet},
    backend::{FakeBackend, FakeFixtureStore},
    domain::{AuditEventId, ErrorCode, LocalUserId},
    mcp::live_orders::{McpLiveOrderContext, handle_live_cancel, handle_live_submit},
    orders::{
        KillSwitch, LocalCandidateLiveWriter, PaperToLiveMigrationChecklist, create_order_preview,
    },
    risk::StaticPolicyRegistry,
};
use std::sync::Arc;

#[tokio::test]
async fn mcp_live_submit_uses_server_side_state_and_replays_idempotency()
-> Result<(), Box<dyn std::error::Error>> {
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let account = live::account_id();
    let order = live::validated_order(account.clone())?;
    let preview = create_order_preview(&order, AuditEventId::new(), None, None)?;
    writer.append_order_preview(&preview, &order).await?;
    let mut approval_service = ApprovalService::default();
    let approval = approval_service.create_approval(
        order.preview_id.clone(),
        account.clone(),
        LocalUserId::from_static("operator"),
        300,
    );
    writer.append_approval(&approval).await?;
    let context = live_context(&writer, account.clone())?;
    let scopes = ScopeSet::local_with_live([ORDERS_LIVE_SUBMIT])?;
    let args = serde_json::json!({
        "account_id": account.as_str(),
        "approval_id": approval.approval_id.as_uuid().to_string(),
        "preview_id": order.preview_id.as_uuid().to_string(),
        "idempotency_key": "mcp-live-submit-key"
    });

    let first = handle_live_submit(&context, &scopes, &args).await?;
    let replayed = handle_live_submit(&context, &scopes, &args).await?;
    assert_eq!(
        first["broker_order_id"],
        "local-candidate-mcp-live-submit-key"
    );
    assert_eq!(first, replayed);
    let pending_live_orders = writer.pending_live_orders().await?;
    assert_eq!(pending_live_orders.len(), 1);
    assert_eq!(
        pending_live_orders[0].broker_order_id.as_str(),
        "local-candidate-mcp-live-submit-key"
    );

    let consumed = handle_live_submit(
        &context,
        &scopes,
        &serde_json::json!({
            "account_id": account.as_str(),
            "approval_id": approval.approval_id.as_uuid().to_string(),
            "preview_id": order.preview_id.as_uuid().to_string(),
            "idempotency_key": "mcp-live-submit-consumed"
        }),
    )
    .await;
    let Err(consumed) = consumed else {
        return Err("fresh submit with consumed approval must refuse".into());
    };
    assert_eq!(consumed.code, ErrorCode::ApprovalConsumed);
    Ok(())
}

#[tokio::test]
async fn mcp_live_cancel_returns_redacted_lifecycle_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let account = live::account_id();
    let context = live_context(&writer, account.clone())?;
    let scopes = ScopeSet::local_with_live([ORDERS_LIVE_CANCEL])?;
    let args = serde_json::json!({
        "account_id": account.as_str(),
        "broker_order_id": "broker-live-1",
        "idempotency_key": "mcp-live-cancel-key"
    });

    let payload = handle_live_cancel(&context, &scopes, &args).await?;
    assert_eq!(payload["broker_order_id"], "broker-live-1");
    assert_eq!(payload["status"], "cancelled");
    assert!(writer.pending_live_orders().await?.is_empty());
    Ok(())
}

#[test]
fn remote_http_registry_can_authorize_live_tools_when_scope_is_allowed() {
    let tool = ibkr_agent_gateway::testing::mcp::find_broker_tool_schema_with_live(
        "ibkr_live_order_submit",
        true,
    );
    assert!(tool.is_some());
}

fn live_context<'a>(
    audit_writer: &'a SqliteAuditWriter,
    account: ibkr_agent_gateway::testing::domain::AccountId,
) -> Result<McpLiveOrderContext<'a>, Box<dyn std::error::Error>> {
    let backend = Box::leak(Box::new(FakeBackend::new(FakeFixtureStore::new(
        "tests/fixtures/cpapi",
    ))));
    let writer = Box::leak(Box::new(LocalCandidateLiveWriter));
    let mut policy = live::live_limit_policy()?;
    policy.max_price_deviation_bps = None;
    policy.max_quote_age_seconds = None;
    let policy_registry = Box::leak(Box::new(StaticPolicyRegistry::single(policy)));

    Ok(McpLiveOrderContext {
        backend,
        audit_writer,
        writer,
        policy_registry,
        live_config: live::live_config(account),
        live_limit_context: live::live_limit_context()?,
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "mcp live test"),
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    })
}
