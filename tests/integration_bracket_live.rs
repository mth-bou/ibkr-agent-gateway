#![cfg(feature = "unstable-internal-test-support")]

use async_trait::async_trait;
use ibkr_agent_gateway::testing::{
    approval::ApprovalService,
    audit::{AuditHmacKey, SqliteAuditWriter},
    auth::{ORDERS_LIVE_SUBMIT, ORDERS_PREVIEW, ScopeSet},
    backend::{FakeBackend, FakeFixtureStore},
    config::LiveTradingConfig,
    domain::{
        AccountId, BrokerOrderId, ErrorCode, GatewayError, LocalUserId, OrderGroupId,
        OrderPreviewId, ValidatedOrder, ValidatedOrderGroup,
    },
    mcp::order_groups::{McpOrderGroupContext, handle_bracket_preview, handle_live_bracket_submit},
    orders::{
        IdempotencyKey, IdempotencyStore, KillSwitch, LiveCancelReceipt, LiveGroupSubmitRequest,
        LiveModifyReceipt, LiveOrderGroupWriter, LiveOrderWriter, LiveSubmitReceipt,
        LocalCandidateLiveGroupWriter, LocalCandidateLiveWriter, OrderModifyFields,
        PaperToLiveMigrationChecklist, SequentialLiveOrderGroupWriter, submit_live_group_order,
    },
};
use serde_json::{Value, json};
use std::sync::Arc;

#[path = "common/live.rs"]
mod live;

#[tokio::test]
async fn live_bracket_submit_refuses_closed_kill_switch() -> Result<(), Box<dyn std::error::Error>>
{
    let account = AccountId::from_static("U1234567");
    let group = ValidatedOrderGroup {
        group_id: OrderGroupId::new(),
        account_id: account.clone(),
        parent: live::validated_order(account.clone())?,
        take_profit: live::validated_order(account.clone())?,
        stop_loss: live::validated_order(account.clone())?,
    };
    let request = LiveGroupSubmitRequest {
        group,
        idempotency_key: IdempotencyKey::new("live-bracket-closed")?,
        live_config: LiveTradingConfig {
            enabled: true,
            allowed_accounts: vec![account],
            risk_policy_id: Some("test".to_string()),
            paper_to_live_checklist_acknowledged: true,
            reconciler_interval_seconds: 5,
        },
        live_scope_granted: true,
        kill_switch: KillSwitch::closed(LocalUserId::from_static("operator"), "test"),
        audit_available: true,
        live_limit_policy: live::live_limit_policy()?,
        live_limit_contexts: vec![
            live::live_limit_context()?,
            live::live_limit_context()?,
            live::live_limit_context()?,
        ],
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    };
    let mut store = IdempotencyStore::default();

    let error = submit_live_group_order(request, &LocalCandidateLiveGroupWriter, &mut store)
        .await
        .err()
        .ok_or("closed kill switch should refuse")?;

    assert_eq!(error.code, ErrorCode::LiveKillSwitchClosed);
    Ok(())
}

#[tokio::test]
async fn mcp_live_bracket_submit_consumes_approvals_and_replays()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let live_writer = LocalCandidateLiveWriter;
    let live_group_writer = SequentialLiveOrderGroupWriter::new(&live_writer);
    let account = AccountId::from_static("DU1234567");
    let context = McpOrderGroupContext {
        backend: &backend,
        audit_writer: &writer,
        live_config: live::live_config(account.clone()),
        live_limit_policy: live::live_limit_policy()?,
        live_group_writer: &live_group_writer,
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "test"),
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    };
    let preview = handle_bracket_preview(
        &context,
        &ScopeSet::local_with_preview([ORDERS_PREVIEW])?,
        &json!({
            "account_id": account.as_str(),
            "symbol": "AAPL",
            "side": "buy",
            "quantity": "1",
            "entry_limit_price": "200",
            "take_profit_limit_price": "205",
            "stop_loss_stop_price": "195"
        }),
    )
    .await?;
    let mut service = ApprovalService::default();
    let parent = approve_leg(&writer, &mut service, &preview, "parent", &account).await?;
    let take_profit = approve_leg(&writer, &mut service, &preview, "take_profit", &account).await?;
    let stop_loss = approve_leg(&writer, &mut service, &preview, "stop_loss", &account).await?;
    let args = json!({
        "account_id": account.as_str(),
        "parent_approval_id": parent,
        "take_profit_approval_id": take_profit,
        "stop_loss_approval_id": stop_loss,
        "idempotency_key": "live-bracket-mcp"
    });

    let first = handle_live_bracket_submit(
        &context,
        &ScopeSet::local_with_live([ORDERS_LIVE_SUBMIT])?,
        &args,
    )
    .await?;
    let replay = handle_live_bracket_submit(
        &context,
        &ScopeSet::local_with_live([ORDERS_LIVE_SUBMIT])?,
        &args,
    )
    .await?;

    assert_eq!(first, replay);
    assert_eq!(first["broker_order_ids"].as_array().map(Vec::len), Some(3));
    for approval_id in [parent, take_profit, stop_loss] {
        let loaded = writer
            .load_approval(&ibkr_agent_gateway::testing::approval::ApprovalId::parse(
                &approval_id,
            )?)
            .await?
            .ok_or("approval should remain persisted")?;
        assert_eq!(
            loaded.status,
            ibkr_agent_gateway::testing::approval::ApprovalStatus::Consumed
        );
    }
    Ok(())
}

async fn approve_leg(
    writer: &SqliteAuditWriter,
    service: &mut ApprovalService,
    preview: &Value,
    leg: &str,
    account: &AccountId,
) -> Result<String, Box<dyn std::error::Error>> {
    let preview_id = OrderPreviewId::parse(
        preview[leg]["preview_id"]
            .as_str()
            .ok_or("preview id should be serialized")?,
    )?;
    let approval = service.create_approval(
        preview_id,
        account.clone(),
        LocalUserId::from_static("operator"),
        300,
    );
    writer.append_approval(&approval).await?;
    Ok(approval.approval_id.as_uuid().to_string())
}

#[tokio::test]
async fn live_bracket_submit_returns_three_broker_ids() -> Result<(), Box<dyn std::error::Error>> {
    let account = AccountId::from_static("U1234567");
    let group = ValidatedOrderGroup {
        group_id: OrderGroupId::new(),
        account_id: account.clone(),
        parent: live::validated_order(account.clone())?,
        take_profit: live::validated_order(account.clone())?,
        stop_loss: live::validated_order(account.clone())?,
    };
    let request = LiveGroupSubmitRequest {
        group,
        idempotency_key: IdempotencyKey::new("live-bracket-ok")?,
        live_config: LiveTradingConfig {
            enabled: true,
            allowed_accounts: vec![account],
            risk_policy_id: Some("test".to_string()),
            paper_to_live_checklist_acknowledged: true,
            reconciler_interval_seconds: 5,
        },
        live_scope_granted: true,
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "test"),
        audit_available: true,
        live_limit_policy: live::live_limit_policy()?,
        live_limit_contexts: vec![
            live::live_limit_context()?,
            live::live_limit_context()?,
            live::live_limit_context()?,
        ],
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    };
    let mut store = IdempotencyStore::default();

    let lifecycle =
        submit_live_group_order(request, &LocalCandidateLiveGroupWriter, &mut store).await?;
    assert_eq!(lifecycle.broker_order_ids.len(), 3);
    assert!(
        lifecycle
            .broker_order_ids
            .iter()
            .all(|id: &BrokerOrderId| id.as_str().starts_with("live-bracket-"))
    );
    Ok(())
}

#[tokio::test]
async fn sequential_live_group_writer_reports_orphaned_parent()
-> Result<(), Box<dyn std::error::Error>> {
    let account = AccountId::from_static("U1234567");
    let group = ValidatedOrderGroup {
        group_id: OrderGroupId::new(),
        account_id: account.clone(),
        parent: live::validated_order(account.clone())?,
        take_profit: live::validated_order(account.clone())?,
        stop_loss: live::validated_order(account)?,
    };
    let writer = SequentialLiveOrderGroupWriter::new(&FailingAfterParentWriter);
    let error = writer
        .submit_live_group(&group, &IdempotencyKey::new("live-bracket-partial")?)
        .await
        .err()
        .ok_or("take-profit failure should surface partial submit cleanup context")?;

    assert_eq!(error.code, ErrorCode::BrokerBackendUnavailable);
    assert!(error.message.contains("live-parent-submitted"));
    assert!(
        error
            .user_action
            .as_deref()
            .unwrap_or_default()
            .contains("live-parent-submitted")
    );
    Ok(())
}

struct FailingAfterParentWriter;

#[async_trait]
impl LiveOrderWriter for FailingAfterParentWriter {
    async fn submit_live(
        &self,
        _order: &ValidatedOrder,
        idempotency_key: &IdempotencyKey,
    ) -> Result<LiveSubmitReceipt, GatewayError> {
        if idempotency_key.as_str().ends_with("-parent") {
            return Ok(LiveSubmitReceipt {
                broker_order_id: BrokerOrderId::from_static("live-parent-submitted"),
                broker_status: Some("Submitted".to_string()),
            });
        }
        Err(GatewayError::new(
            ErrorCode::BrokerBackendUnavailable,
            "simulated later bracket leg failure",
            true,
            Some("retry after checking broker state".to_string()),
        ))
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

    async fn modify_live(
        &self,
        _account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        _changes: &OrderModifyFields,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<LiveModifyReceipt, GatewayError> {
        Ok(LiveModifyReceipt {
            broker_order_id: broker_order_id.clone(),
            accepted: true,
            broker_status: Some("Modified".to_string()),
        })
    }
}
