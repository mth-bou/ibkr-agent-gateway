#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    approval::{ApprovalId, ApprovalService, ApprovalStatus},
    audit::{AuditHmacKey, SqliteAuditWriter},
    auth::{ORDERS_PAPER_SUBMIT, ORDERS_PREVIEW, ScopeSet},
    backend::{FakeBackend, FakeFixtureStore},
    config::LiveTradingConfig,
    domain::{LocalUserId, OrderPreviewId},
    mcp::order_groups::{
        McpOrderGroupContext, handle_bracket_preview, handle_paper_bracket_submit,
    },
    orders::{KillSwitch, LocalCandidateLiveGroupWriter, PaperToLiveMigrationChecklist},
    risk::LiveLimitPolicy,
};
use serde_json::{Value, json};
use std::sync::Arc;

#[tokio::test]
async fn mcp_paper_bracket_submit_uses_approved_server_previews()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let context = context(&backend, &writer);
    let preview_scopes = ScopeSet::local_with_preview([ORDERS_PREVIEW])?;
    let preview = handle_bracket_preview(
        &context,
        &preview_scopes,
        &json!({
            "account_id": "DU1234567",
            "symbol": "AAPL",
            "side": "buy",
            "quantity": "1",
            "entry_limit_price": "100",
            "take_profit_limit_price": "110",
            "stop_loss_stop_price": "95"
        }),
    )
    .await?;
    let mut service = ApprovalService::default();
    let parent = approve_leg(&writer, &mut service, &preview, "parent").await?;
    let take_profit = approve_leg(&writer, &mut service, &preview, "take_profit").await?;
    let stop_loss = approve_leg(&writer, &mut service, &preview, "stop_loss").await?;
    let submit_scopes = ScopeSet::local_with_paper([ORDERS_PAPER_SUBMIT])?;

    let payload = handle_paper_bracket_submit(
        &context,
        &submit_scopes,
        &json!({
            "account_id": "DU1234567",
            "parent_approval_id": parent,
            "take_profit_approval_id": take_profit,
            "stop_loss_approval_id": stop_loss,
            "idempotency_key": "paper-bracket-1"
        }),
    )
    .await?;

    assert_eq!(payload["account_id"], "DU1234567");
    assert_eq!(payload["status"], "submitted");
    assert_eq!(
        payload["broker_order_ids"].as_array().map(Vec::len),
        Some(3)
    );
    Ok(())
}

#[tokio::test]
async fn mcp_paper_bracket_submit_replays_after_approvals_are_consumed()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let context = context(&backend, &writer);
    let preview = preview_bracket(&context, "100", "110", "95").await?;
    let mut service = ApprovalService::default();
    let parent = approve_leg(&writer, &mut service, &preview, "parent").await?;
    let take_profit = approve_leg(&writer, &mut service, &preview, "take_profit").await?;
    let stop_loss = approve_leg(&writer, &mut service, &preview, "stop_loss").await?;
    let submit_scopes = ScopeSet::local_with_paper([ORDERS_PAPER_SUBMIT])?;
    let args = json!({
        "account_id": "DU1234567",
        "parent_approval_id": parent,
        "take_profit_approval_id": take_profit,
        "stop_loss_approval_id": stop_loss,
        "idempotency_key": "paper-bracket-replay"
    });

    let first = handle_paper_bracket_submit(&context, &submit_scopes, &args).await?;
    for approval_id in [
        args["parent_approval_id"]
            .as_str()
            .ok_or("missing parent id")?,
        args["take_profit_approval_id"]
            .as_str()
            .ok_or("missing take-profit id")?,
        args["stop_loss_approval_id"]
            .as_str()
            .ok_or("missing stop-loss id")?,
    ] {
        let approval = writer
            .load_approval(&ApprovalId::parse(approval_id)?)
            .await?
            .ok_or("approval should exist")?;
        assert_eq!(approval.status, ApprovalStatus::Consumed);
        writer.mark_approval_consumed(&approval).await?;
    }

    let replay = handle_paper_bracket_submit(&context, &submit_scopes, &args).await?;

    assert_eq!(first, replay);
    Ok(())
}

#[tokio::test]
async fn mcp_paper_bracket_submit_refuses_approvals_from_different_groups()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let context = context(&backend, &writer);
    let first_preview = preview_bracket(&context, "100", "110", "95").await?;
    let second_preview = preview_bracket(&context, "101", "111", "96").await?;
    let mut service = ApprovalService::default();
    let parent = approve_leg(&writer, &mut service, &first_preview, "parent").await?;
    let take_profit = approve_leg(&writer, &mut service, &second_preview, "take_profit").await?;
    let stop_loss = approve_leg(&writer, &mut service, &second_preview, "stop_loss").await?;
    let submit_scopes = ScopeSet::local_with_paper([ORDERS_PAPER_SUBMIT])?;

    let error = handle_paper_bracket_submit(
        &context,
        &submit_scopes,
        &json!({
            "account_id": "DU1234567",
            "parent_approval_id": parent,
            "take_profit_approval_id": take_profit,
            "stop_loss_approval_id": stop_loss,
            "idempotency_key": "paper-bracket-mixed"
        }),
    )
    .await
    .err()
    .ok_or("mixed bracket approvals should be refused")?;

    assert_eq!(
        error.code,
        ibkr_agent_gateway::testing::domain::ErrorCode::ApprovalPreviewMismatch
    );
    Ok(())
}

fn context<'a>(
    backend: &'a FakeBackend,
    writer: &'a SqliteAuditWriter,
) -> McpOrderGroupContext<'a> {
    McpOrderGroupContext {
        backend,
        audit_writer: writer,
        live_config: LiveTradingConfig::default(),
        live_limit_policy: LiveLimitPolicy::default(),
        live_group_writer: &LocalCandidateLiveGroupWriter,
        kill_switch: KillSwitch::closed(LocalUserId::from_static("operator"), "test"),
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    }
}

async fn approve_leg(
    writer: &SqliteAuditWriter,
    service: &mut ApprovalService,
    preview: &Value,
    leg: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let preview_id = OrderPreviewId::parse(
        preview[leg]["preview_id"]
            .as_str()
            .ok_or("preview id should be serialized")?,
    )?;
    let approval = service.create_approval(
        preview_id,
        ibkr_agent_gateway::testing::domain::AccountId::from_static("DU1234567"),
        LocalUserId::from_static("operator"),
        300,
    );
    writer.append_approval(&approval).await?;
    Ok(approval.approval_id.as_uuid().to_string())
}

async fn preview_bracket(
    context: &McpOrderGroupContext<'_>,
    entry_limit_price: &str,
    take_profit_limit_price: &str,
    stop_loss_stop_price: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(handle_bracket_preview(
        context,
        &ScopeSet::local_with_preview([ORDERS_PREVIEW])?,
        &json!({
            "account_id": "DU1234567",
            "symbol": "AAPL",
            "side": "buy",
            "quantity": "1",
            "entry_limit_price": entry_limit_price,
            "take_profit_limit_price": take_profit_limit_price,
            "stop_loss_stop_price": stop_loss_stop_price
        }),
    )
    .await?)
}
