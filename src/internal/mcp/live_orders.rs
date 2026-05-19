//! MCP handlers for explicit live order tools.

use super::{
    order_workflows::parse_modify_fields,
    tools::orders_live::{LIVE_ORDER_CANCEL_TOOL, LIVE_ORDER_MODIFY_TOOL, LIVE_ORDER_SUBMIT_TOOL},
};
use crate::internal::{
    audit::{
        OrderIdempotencyOperation, OrderIdempotencyRecoveryContext, OrderIdempotencyWorkflow,
        SqliteAuditWriter,
    },
    auth::{ORDERS_LIVE_CANCEL, ORDERS_LIVE_MODIFY, ORDERS_LIVE_SUBMIT, ScopeSet},
    backend::IbkrBackend,
    config::LiveTradingConfig,
    domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError, OrderPreviewId},
    mcp::enforce_scope,
    orders::{
        IdempotencyKey, KillSwitch, LiveCancelRequest, LiveModifyRequest, LiveOrderWriter,
        LiveSubmitRequest, OrderModifyFields, PaperToLiveMigrationChecklist,
        cancel_live_order_without_local_idempotency, handle_pending_order_error,
        modify_live_order_without_local_idempotency, stable_request_hash,
        submit_live_order_without_local_idempotency,
    },
    risk::{LivePolicyRegistry, apply_live_rate_counters, live_limit_context_for_order},
};
use serde::Serialize;
use serde_json::Value;

/// Server-owned dependencies for live MCP order handlers.
pub struct McpLiveOrderContext<'a> {
    /// Broker read backend used for market snapshots and recovery lookups.
    pub backend: &'a dyn IbkrBackend,
    /// Durable audit and workflow state.
    pub audit_writer: &'a SqliteAuditWriter,
    /// Server-wired live broker writer.
    pub writer: &'a dyn LiveOrderWriter,
    /// Server-side live policy registry.
    pub policy_registry: &'a dyn LivePolicyRegistry,
    /// Server-owned live trading config.
    pub live_config: LiveTradingConfig,
    /// Server kill switch state.
    pub kill_switch: KillSwitch,
    /// Server-owned migration checklist acknowledgement.
    pub migration_checklist: PaperToLiveMigrationChecklist,
}

/// Dispatches one explicit live MCP tool call after transport-level auth.
pub async fn handle_live_order_tool(
    context: &McpLiveOrderContext<'_>,
    scopes: &ScopeSet,
    tool_name: &str,
    args: &Value,
) -> Result<Value, GatewayError> {
    match tool_name {
        LIVE_ORDER_SUBMIT_TOOL => handle_live_submit(context, scopes, args).await,
        LIVE_ORDER_CANCEL_TOOL => handle_live_cancel(context, scopes, args).await,
        LIVE_ORDER_MODIFY_TOOL => handle_live_modify(context, scopes, args).await,
        _ => Err(GatewayError::new(
            ErrorCode::ReadonlyWriteForbidden,
            format!("MCP tool {tool_name} is not a live order tool"),
            false,
            Some("Use a registered live order MCP tool".to_string()),
        )),
    }
}

/// Handles `ibkr_live_order_submit`.
pub async fn handle_live_submit(
    context: &McpLiveOrderContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_LIVE_SUBMIT)?;
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let approval_id =
        crate::internal::approval::ApprovalId::parse(arg_string(args, "approval_id")?)?;
    let preview_id = OrderPreviewId::parse(arg_string(args, "preview_id")?)?;
    let idempotency_key = IdempotencyKey::new(arg_string(args, "idempotency_key")?)?;
    let request_hash = stable_request_hash(
        "mcp.live.submit",
        &LiveSubmitMcpFingerprint {
            account_id: account_id.as_str(),
            approval_id: approval_id.as_uuid().to_string(),
            preview_id: preview_id.as_uuid().to_string(),
        },
    )?;
    if let Some(payload) = context
        .audit_writer
        .replay_order_idempotency(&idempotency_key, &request_hash)
        .await?
    {
        return Ok(payload);
    }

    let approval = context
        .audit_writer
        .load_approval(&approval_id)
        .await?
        .ok_or_else(missing_approval)?;
    if approval.preview_id != preview_id {
        return Err(GatewayError::new(
            ErrorCode::ApprovalPreviewMismatch,
            "Approval preview id does not match requested preview id",
            false,
            Some("Submit the approved preview id".to_string()),
        ));
    }
    let preview_record = context
        .audit_writer
        .load_order_preview(&preview_id)
        .await?
        .ok_or_else(missing_preview)?;
    if preview_record.validated_order.account_id != account_id {
        return Err(GatewayError::new(
            ErrorCode::InputUnauthorizedAccount,
            "Requested account does not match the preview account",
            false,
            Some("Use the account id from the approved preview".to_string()),
        ));
    }

    let market_snapshot = context
        .backend
        .market_snapshot(&preview_record.validated_order.contract_id)
        .await?;
    let mut live_limit_context =
        live_limit_context_for_order(&preview_record.validated_order, Some(market_snapshot))?;
    if let Some(policy_id) = context.live_config.risk_policy_id.as_deref() {
        let live_policy = context.policy_registry.load_policy(policy_id).await?;
        apply_live_rate_counters(
            context.audit_writer,
            &account_id,
            &live_policy,
            &mut live_limit_context,
        )
        .await?;
    }
    let request = LiveSubmitRequest {
        order: preview_record.validated_order,
        approval,
        idempotency_key: idempotency_key.clone(),
        live_config: context.live_config.clone(),
        live_scope_granted: true,
        live_limit_context,
        kill_switch: context.kill_switch.clone(),
        audit_available: true,
        migration_checklist: context.migration_checklist.clone(),
    };
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Live,
        operation: OrderIdempotencyOperation::Submit,
        account_id: request.order.account_id.clone(),
        broker_order_id: None,
    };
    context
        .audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;

    let result = match submit_live_order_without_local_idempotency(
        request,
        context.writer,
        context.policy_registry,
    )
    .await
    {
        Ok(result) => result,
        Err(error) => {
            handle_pending_order_error(
                context.audit_writer,
                &idempotency_key,
                &request_hash,
                &error,
            )
            .await?;
            return Err(error);
        }
    };
    let payload = serde_json::to_value(&result.lifecycle).map_err(output_error)?;
    context
        .audit_writer
        .complete_live_order_workflow(
            &idempotency_key,
            &request_hash,
            &payload,
            &result.lifecycle,
            std::slice::from_ref(&result.consumed_approval),
        )
        .await?;
    Ok(payload)
}

/// Handles `ibkr_live_order_cancel`.
pub async fn handle_live_cancel(
    context: &McpLiveOrderContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_LIVE_CANCEL)?;
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let broker_order_id = parse_broker_order_id(arg_string(args, "broker_order_id")?)?;
    let idempotency_key = IdempotencyKey::new(arg_string(args, "idempotency_key")?)?;
    let request_hash = stable_request_hash(
        "mcp.live.cancel",
        &LiveCancelMcpFingerprint {
            account_id: account_id.as_str(),
            broker_order_id: broker_order_id.as_str(),
        },
    )?;
    if let Some(payload) = context
        .audit_writer
        .replay_order_idempotency(&idempotency_key, &request_hash)
        .await?
    {
        return Ok(payload);
    }

    let request = LiveCancelRequest {
        account_id: account_id.clone(),
        broker_order_id: broker_order_id.clone(),
        idempotency_key: idempotency_key.clone(),
        live_config: context.live_config.clone(),
        live_scope_granted: true,
        kill_switch: context.kill_switch.clone(),
        audit_available: true,
        migration_checklist: context.migration_checklist.clone(),
    };
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Live,
        operation: OrderIdempotencyOperation::Cancel,
        account_id,
        broker_order_id: Some(broker_order_id),
    };
    context
        .audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;

    let result = match cancel_live_order_without_local_idempotency(request, context.writer).await {
        Ok(result) => result,
        Err(error) => {
            handle_pending_order_error(
                context.audit_writer,
                &idempotency_key,
                &request_hash,
                &error,
            )
            .await?;
            return Err(error);
        }
    };
    let payload = serde_json::to_value(&result.lifecycle).map_err(output_error)?;
    context
        .audit_writer
        .complete_live_order_workflow(
            &idempotency_key,
            &request_hash,
            &payload,
            &result.lifecycle,
            &[],
        )
        .await?;
    Ok(payload)
}

/// Handles `ibkr_live_order_modify`.
pub async fn handle_live_modify(
    context: &McpLiveOrderContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_LIVE_MODIFY)?;
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let broker_order_id = parse_broker_order_id(arg_string(args, "broker_order_id")?)?;
    let approval_id =
        crate::internal::approval::ApprovalId::parse(arg_string(args, "approval_id")?)?;
    let preview_id = OrderPreviewId::parse(arg_string(args, "preview_id")?)?;
    let idempotency_key = IdempotencyKey::new(arg_string(args, "idempotency_key")?)?;
    let changes = parse_modify_fields(args)?;
    let request_hash = stable_request_hash(
        "mcp.live.modify",
        &LiveModifyMcpFingerprint {
            account_id: account_id.as_str(),
            broker_order_id: broker_order_id.as_str(),
            approval_id: approval_id.as_uuid().to_string(),
            preview_id: preview_id.as_uuid().to_string(),
            changes: &changes,
        },
    )?;
    if let Some(payload) = context
        .audit_writer
        .replay_order_idempotency(&idempotency_key, &request_hash)
        .await?
    {
        return Ok(payload);
    }

    let approval = context
        .audit_writer
        .load_approval(&approval_id)
        .await?
        .ok_or_else(missing_approval)?;
    if approval.preview_id != preview_id {
        return Err(GatewayError::new(
            ErrorCode::ApprovalPreviewMismatch,
            "Approval preview id does not match requested replacement preview id",
            false,
            Some("Modify using the approved replacement preview id".to_string()),
        ));
    }
    let preview_record = context
        .audit_writer
        .load_order_preview(&preview_id)
        .await?
        .ok_or_else(missing_preview)?;
    if preview_record.validated_order.account_id != account_id {
        return Err(GatewayError::new(
            ErrorCode::InputUnauthorizedAccount,
            "Requested account does not match the replacement preview account",
            false,
            Some("Use the account id from the approved replacement preview".to_string()),
        ));
    }
    let market_snapshot = context
        .backend
        .market_snapshot(&preview_record.validated_order.contract_id)
        .await?;
    let mut live_limit_context =
        live_limit_context_for_order(&preview_record.validated_order, Some(market_snapshot))?;
    if let Some(policy_id) = context.live_config.risk_policy_id.as_deref() {
        let live_policy = context.policy_registry.load_policy(policy_id).await?;
        apply_live_rate_counters(
            context.audit_writer,
            &account_id,
            &live_policy,
            &mut live_limit_context,
        )
        .await?;
    }

    let request = LiveModifyRequest {
        account_id: account_id.clone(),
        broker_order_id: broker_order_id.clone(),
        changes,
        approved_order: preview_record.validated_order,
        approval,
        idempotency_key: idempotency_key.clone(),
        live_config: context.live_config.clone(),
        live_scope_granted: true,
        kill_switch: context.kill_switch.clone(),
        audit_available: true,
        live_limit_context,
        migration_checklist: context.migration_checklist.clone(),
    };
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Live,
        operation: OrderIdempotencyOperation::Modify,
        account_id,
        broker_order_id: Some(broker_order_id),
    };
    context
        .audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;

    let result = match modify_live_order_without_local_idempotency(
        request,
        context.writer,
        context.policy_registry,
    )
    .await
    {
        Ok(result) => result,
        Err(error) => {
            handle_pending_order_error(
                context.audit_writer,
                &idempotency_key,
                &request_hash,
                &error,
            )
            .await?;
            return Err(error);
        }
    };
    let payload = serde_json::to_value(&result.lifecycle).map_err(output_error)?;
    context
        .audit_writer
        .complete_live_order_workflow(
            &idempotency_key,
            &request_hash,
            &payload,
            &result.lifecycle,
            std::slice::from_ref(&result.consumed_approval),
        )
        .await?;
    Ok(payload)
}

#[derive(Serialize)]
struct LiveSubmitMcpFingerprint<'a> {
    account_id: &'a str,
    approval_id: String,
    preview_id: String,
}

#[derive(Serialize)]
struct LiveCancelMcpFingerprint<'a> {
    account_id: &'a str,
    broker_order_id: &'a str,
}

#[derive(Serialize)]
struct LiveModifyMcpFingerprint<'a> {
    account_id: &'a str,
    broker_order_id: &'a str,
    approval_id: String,
    preview_id: String,
    changes: &'a OrderModifyFields,
}

fn arg_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, GatewayError> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            GatewayError::new(
                ErrorCode::ConfigInvalid,
                format!("Missing MCP argument: {key}"),
                false,
                Some("Send a valid MCP live order request".to_string()),
            )
        })
}

fn parse_account_id(account_id: &str) -> Result<AccountId, GatewayError> {
    AccountId::new(account_id).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::InputMissingAccount,
            "Account id is required",
            false,
            Some("Select one account explicitly".to_string()),
        )
    })
}

fn parse_broker_order_id(broker_order_id: &str) -> Result<BrokerOrderId, GatewayError> {
    BrokerOrderId::new(broker_order_id).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Broker order id is required",
            false,
            Some("Provide a broker order id".to_string()),
        )
    })
}

fn missing_approval() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Live submit requires an existing approval record",
        false,
        Some("Create an approval for the preview before live submit".to_string()),
    )
}

fn missing_preview() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Live submit requires the approved preview to be present",
        false,
        Some("Create a fresh preview and approval before live submit".to_string()),
    )
}

fn output_error(_error: serde_json::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::OutputUnsafe,
        "Failed to serialize live MCP result",
        false,
        Some("Retry the MCP request".to_string()),
    )
}
