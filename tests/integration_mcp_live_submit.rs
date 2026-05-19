#![cfg(feature = "unstable-internal-test-support")]

#[path = "common/live.rs"]
mod live;

use async_trait::async_trait;
use ibkr_agent_gateway::testing::{
    approval::ApprovalService,
    audit::{AuditHmacKey, AuditTailRequest, SqliteAuditWriter},
    auth::{ORDERS_LIVE_CANCEL, ORDERS_LIVE_MODIFY, ORDERS_LIVE_SUBMIT, ScopeSet},
    backend::{FakeBackend, FakeFixtureStore},
    domain::{
        AccountId, AssetClass, AuditEventId, BrokerOrderId, CurrencyCode, ErrorCode, GatewayError,
        LocalUserId, Money, ValidatedOrder,
    },
    mcp::live_orders::{
        McpLiveOrderContext, handle_live_cancel, handle_live_modify, handle_live_submit,
    },
    orders::{
        IdempotencyKey, KillSwitch, LiveCancelReceipt, LiveModifyReceipt, LiveOrderLifecycleRecord,
        LiveOrderLifecycleStatus, LiveOrderWriter, LiveSubmitReceipt, LocalCandidateLiveWriter,
        OrderModifyFields, PaperToLiveMigrationChecklist, create_order_preview,
    },
    risk::StaticPolicyRegistry,
};
use rust_decimal::Decimal;
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
    let tail = writer.tail_verified(AuditTailRequest::new(10)).await?;
    assert!(
        tail.events
            .iter()
            .all(|record| record.event.tool_name.as_deref() != Some("ibkr_live_order_submit")),
        "live handlers must delegate MCP tool audit to the transport layer"
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
async fn mcp_live_submit_uses_preview_instrument_context_for_policy()
-> Result<(), Box<dyn std::error::Error>> {
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let account = live::account_id();
    let mut order = live::validated_order(account.clone())?;
    order.symbol = Some("MSFT".to_string());
    order.asset_class = Some(AssetClass::Etf);
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

    let mut policy = live::live_limit_policy()?;
    policy.allowed_symbols = vec!["MSFT".to_string()];
    policy.allowed_asset_classes = vec![AssetClass::Etf];
    policy.max_price_deviation_bps = None;
    policy.max_quote_age_seconds = None;
    let context = live_context_with_policy(
        &writer,
        account.clone(),
        policy,
        Box::new(LocalCandidateLiveWriter),
    )?;
    let scopes = ScopeSet::local_with_live([ORDERS_LIVE_SUBMIT])?;

    let payload = handle_live_submit(
        &context,
        &scopes,
        &serde_json::json!({
            "account_id": account.as_str(),
            "approval_id": approval.approval_id.as_uuid().to_string(),
            "preview_id": order.preview_id.as_uuid().to_string(),
            "idempotency_key": "mcp-live-msft-submit"
        }),
    )
    .await?;

    assert_eq!(
        payload["broker_order_id"],
        "local-candidate-mcp-live-msft-submit"
    );
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

#[tokio::test]
async fn mcp_live_cancel_keeps_pending_cancel_in_backlog() -> Result<(), Box<dyn std::error::Error>>
{
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let account = live::account_id();
    let live_writer = PendingCancelWriter;
    let context = live_context_with_live_writer(&writer, account.clone(), &live_writer)?;
    let scopes = ScopeSet::local_with_live([ORDERS_LIVE_CANCEL])?;
    let args = serde_json::json!({
        "account_id": account.as_str(),
        "broker_order_id": "broker-live-pending",
        "idempotency_key": "mcp-live-cancel-pending-key"
    });

    let payload = handle_live_cancel(&context, &scopes, &args).await?;

    assert_eq!(payload["status"], "pending_cancel");
    let pending_live_orders = writer.pending_live_orders().await?;
    assert_eq!(pending_live_orders.len(), 1);
    assert_eq!(
        pending_live_orders[0].broker_order_id.as_str(),
        "broker-live-pending"
    );
    assert_eq!(
        pending_live_orders[0].last_status,
        LiveOrderLifecycleStatus::PendingCancel
    );
    Ok(())
}

#[tokio::test]
async fn mcp_live_modify_uses_live_gates_and_replays_idempotency()
-> Result<(), Box<dyn std::error::Error>> {
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let account = live::account_id();
    let mut replacement_order = live::validated_order(account.clone())?;
    replacement_order.limit_price = Some(Money {
        amount: Decimal::new(12375, 2),
        currency: CurrencyCode::new("USD").ok_or("static currency rejected")?,
    });
    let preview = create_order_preview(&replacement_order, AuditEventId::new(), None, None)?;
    writer
        .append_order_preview(&preview, &replacement_order)
        .await?;
    let mut approval_service = ApprovalService::default();
    let approval = approval_service.create_approval(
        replacement_order.preview_id.clone(),
        account.clone(),
        LocalUserId::from_static("operator"),
        300,
    );
    writer.append_approval(&approval).await?;
    let context = live_context(&writer, account.clone())?;
    let scopes = ScopeSet::local_with_live([ORDERS_LIVE_MODIFY])?;
    let args = serde_json::json!({
        "account_id": account.as_str(),
        "broker_order_id": "live-order-1",
        "approval_id": approval.approval_id.as_uuid().to_string(),
        "preview_id": replacement_order.preview_id.as_uuid().to_string(),
        "idempotency_key": "live-modify-1",
        "limit_price": "123.75"
    });

    let first = handle_live_modify(&context, &scopes, &args).await?;
    let replay = handle_live_modify(&context, &scopes, &args).await?;

    assert_eq!(first, replay);
    assert_eq!(first["account_id"], account.as_str());
    assert_eq!(first["broker_order_id"], "live-order-1");
    assert_eq!(first["notional"]["amount"], "123.75");
    assert_eq!(
        serde_json::from_value::<LiveOrderLifecycleStatus>(first["status"].clone())?,
        LiveOrderLifecycleStatus::Open
    );
    Ok(())
}

#[tokio::test]
async fn mcp_live_modify_invalid_changes_do_not_poison_idempotency_key()
-> Result<(), Box<dyn std::error::Error>> {
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let account = live::account_id();
    let mut replacement_order = live::validated_order(account.clone())?;
    replacement_order.limit_price = Some(Money {
        amount: Decimal::new(12375, 2),
        currency: CurrencyCode::new("USD").ok_or("static currency rejected")?,
    });
    let preview = create_order_preview(&replacement_order, AuditEventId::new(), None, None)?;
    writer
        .append_order_preview(&preview, &replacement_order)
        .await?;
    let mut approval_service = ApprovalService::default();
    let approval = approval_service.create_approval(
        replacement_order.preview_id.clone(),
        account.clone(),
        LocalUserId::from_static("operator"),
        300,
    );
    writer.append_approval(&approval).await?;
    let context = live_context(&writer, account.clone())?;
    let scopes = ScopeSet::local_with_live([ORDERS_LIVE_MODIFY])?;
    let invalid_args = serde_json::json!({
        "account_id": account.as_str(),
        "broker_order_id": "live-order-validate-first",
        "approval_id": approval.approval_id.as_uuid().to_string(),
        "preview_id": replacement_order.preview_id.as_uuid().to_string(),
        "idempotency_key": "live-modify-validation-retry"
    });

    let error = handle_live_modify(&context, &scopes, &invalid_args)
        .await
        .err()
        .ok_or("empty live modify should be refused")?;
    assert_eq!(error.code, ErrorCode::OrderValidationFailed);

    let valid_args = serde_json::json!({
        "account_id": account.as_str(),
        "broker_order_id": "live-order-validate-first",
        "approval_id": approval.approval_id.as_uuid().to_string(),
        "preview_id": replacement_order.preview_id.as_uuid().to_string(),
        "idempotency_key": "live-modify-validation-retry",
        "limit_price": "123.75"
    });
    let payload = handle_live_modify(&context, &scopes, &valid_args).await?;

    assert_eq!(payload["broker_order_id"], "live-order-validate-first");
    assert_eq!(payload["notional"]["amount"], "123.75");
    Ok(())
}

#[tokio::test]
async fn mcp_live_submit_uses_audit_backed_rate_counters() -> Result<(), Box<dyn std::error::Error>>
{
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
    let prior_lifecycle = LiveOrderLifecycleRecord {
        account_id: account.clone(),
        broker_order_id: ibkr_agent_gateway::testing::domain::BrokerOrderId::from_static(
            "prior-live-1",
        ),
        status: LiveOrderLifecycleStatus::Submitted,
        notional: None,
        execution_correlation: None,
        updated_at: time::OffsetDateTime::now_utc(),
    };
    writer
        .insert_order_idempotency(
            &IdempotencyKey::new("prior-live-rate-key")?,
            "prior-live-rate-hash",
            &serde_json::to_value(&prior_lifecycle)?,
        )
        .await?;

    let mut policy = live::live_limit_policy()?;
    if let Some(session_limit) = &mut policy.session_limit {
        session_limit.max_orders_per_session = 1;
    }
    policy.max_price_deviation_bps = None;
    policy.max_quote_age_seconds = None;
    let context = live_context_with_policy(
        &writer,
        account.clone(),
        policy,
        Box::new(LocalCandidateLiveWriter),
    )?;
    let scopes = ScopeSet::local_with_live([ORDERS_LIVE_SUBMIT])?;
    let result = handle_live_submit(
        &context,
        &scopes,
        &serde_json::json!({
            "account_id": account.as_str(),
            "approval_id": approval.approval_id.as_uuid().to_string(),
            "preview_id": order.preview_id.as_uuid().to_string(),
            "idempotency_key": "mcp-live-rate-limited"
        }),
    )
    .await;
    let Err(error) = result else {
        return Err("server-side rate counters should refuse the second live submit".into());
    };
    assert_eq!(error.code, ErrorCode::LiveLimitRefused);
    Ok(())
}

#[tokio::test]
async fn mcp_live_submit_uses_audit_backed_session_notional()
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
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err("static currency rejected".into());
    };
    let prior_lifecycle = LiveOrderLifecycleRecord {
        account_id: account.clone(),
        broker_order_id: ibkr_agent_gateway::testing::domain::BrokerOrderId::from_static(
            "prior-live-notional",
        ),
        status: LiveOrderLifecycleStatus::Submitted,
        notional: Some(Money {
            amount: Decimal::new(4_950, 0),
            currency,
        }),
        execution_correlation: None,
        updated_at: time::OffsetDateTime::now_utc(),
    };
    writer
        .insert_order_idempotency(
            &IdempotencyKey::new("prior-live-notional-key")?,
            "prior-live-notional-hash",
            &serde_json::to_value(&prior_lifecycle)?,
        )
        .await?;

    let mut policy = live::live_limit_policy()?;
    if let Some(session_limit) = &mut policy.session_limit {
        session_limit.max_orders_per_session = 20;
        session_limit.max_session_notional = Some(Money {
            amount: Decimal::new(5_000, 0),
            currency: CurrencyCode::new("USD").ok_or("static currency rejected")?,
        });
    }
    policy.max_price_deviation_bps = None;
    policy.max_quote_age_seconds = None;
    let context = live_context_with_policy(
        &writer,
        account.clone(),
        policy,
        Box::new(LocalCandidateLiveWriter),
    )?;
    let scopes = ScopeSet::local_with_live([ORDERS_LIVE_SUBMIT])?;
    let result = handle_live_submit(
        &context,
        &scopes,
        &serde_json::json!({
            "account_id": account.as_str(),
            "approval_id": approval.approval_id.as_uuid().to_string(),
            "preview_id": order.preview_id.as_uuid().to_string(),
            "idempotency_key": "mcp-live-notional-limited"
        }),
    )
    .await;

    let Err(error) = result else {
        return Err("server-side session notional should refuse the live submit".into());
    };
    assert_eq!(error.code, ErrorCode::LiveLimitRefused);
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
    let mut policy = live::live_limit_policy()?;
    policy.max_price_deviation_bps = None;
    policy.max_quote_age_seconds = None;
    live_context_with_policy(
        audit_writer,
        account,
        policy,
        Box::new(LocalCandidateLiveWriter),
    )
}

fn live_context_with_live_writer<'a>(
    audit_writer: &'a SqliteAuditWriter,
    account: ibkr_agent_gateway::testing::domain::AccountId,
    writer: &'a dyn LiveOrderWriter,
) -> Result<McpLiveOrderContext<'a>, Box<dyn std::error::Error>> {
    let mut policy = live::live_limit_policy()?;
    policy.max_price_deviation_bps = None;
    policy.max_quote_age_seconds = None;
    let backend = Box::leak(Box::new(FakeBackend::new(FakeFixtureStore::new(
        "tests/fixtures/cpapi",
    ))));
    let policy_registry = Box::leak(Box::new(StaticPolicyRegistry::single(policy)));

    Ok(McpLiveOrderContext {
        backend,
        audit_writer,
        writer,
        policy_registry,
        live_config: live::live_config(account),
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "mcp live test"),
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    })
}

fn live_context_with_policy<'a>(
    audit_writer: &'a SqliteAuditWriter,
    account: ibkr_agent_gateway::testing::domain::AccountId,
    policy: ibkr_agent_gateway::testing::risk::LiveLimitPolicy,
    writer: Box<dyn LiveOrderWriter>,
) -> Result<McpLiveOrderContext<'a>, Box<dyn std::error::Error>> {
    let backend = Box::leak(Box::new(FakeBackend::new(FakeFixtureStore::new(
        "tests/fixtures/cpapi",
    ))));
    let writer = Box::leak(writer);
    let policy_registry = Box::leak(Box::new(StaticPolicyRegistry::single(policy)));

    Ok(McpLiveOrderContext {
        backend,
        audit_writer,
        writer,
        policy_registry,
        live_config: live::live_config(account),
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "mcp live test"),
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    })
}

struct PendingCancelWriter;

#[async_trait]
impl LiveOrderWriter for PendingCancelWriter {
    async fn submit_live(
        &self,
        _order: &ValidatedOrder,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<LiveSubmitReceipt, GatewayError> {
        Err(GatewayError::new(
            ErrorCode::BrokerResponseInvalid,
            "submit is not used in this test",
            false,
            None,
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
            broker_status: Some("PendingCancel".to_string()),
        })
    }

    async fn modify_live(
        &self,
        _account_id: &AccountId,
        _broker_order_id: &BrokerOrderId,
        _changes: &OrderModifyFields,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<LiveModifyReceipt, GatewayError> {
        Err(GatewayError::new(
            ErrorCode::BrokerResponseInvalid,
            "modify is not used in this test",
            false,
            None,
        ))
    }
}
