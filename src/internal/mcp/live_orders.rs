//! MCP handlers for explicit live order tools.

use super::tools::orders_live::{LIVE_ORDER_CANCEL_TOOL, LIVE_ORDER_SUBMIT_TOOL};
use crate::internal::{
    audit::{
        OrderIdempotencyOperation, OrderIdempotencyRecoveryContext, OrderIdempotencyWorkflow,
        SqliteAuditWriter,
    },
    auth::{ORDERS_LIVE_CANCEL, ORDERS_LIVE_SUBMIT, ScopeSet},
    backend::IbkrBackend,
    config::LiveTradingConfig,
    domain::{AccountId, BrokerOrderId, ErrorCode, GatewayError, OrderPreviewId},
    mcp::{build_mcp_tool_event, enforce_scope},
    orders::{
        IdempotencyKey, IdempotencyStore, KillSwitch, LiveCancelRequest, LiveOrderWriter,
        LiveSubmitRequest, PaperToLiveMigrationChecklist, cancel_live_order, stable_request_hash,
        submit_live_order,
    },
    risk::{LiveLimitContext, LivePolicyRegistry},
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
    /// Base live limit context owned by the server runtime.
    pub live_limit_context: LiveLimitContext,
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

    let mut live_limit_context = context.live_limit_context.clone();
    live_limit_context.market_snapshot = Some(
        context
            .backend
            .market_snapshot(&preview_record.validated_order.contract_id)
            .await?,
    );
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

    let mut idempotency_store = IdempotencyStore::default();
    let result = match submit_live_order(
        request,
        context.writer,
        context.policy_registry,
        &mut idempotency_store,
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
            record_live_tool_audit(
                context.audit_writer,
                LIVE_ORDER_SUBMIT_TOOL,
                ORDERS_LIVE_SUBMIT,
                &Err(error.clone()),
            )
            .await?;
            return Err(error);
        }
    };
    let payload = serde_json::to_value(&result.lifecycle).map_err(output_error)?;
    context
        .audit_writer
        .insert_order_idempotency(&idempotency_key, &request_hash, &payload)
        .await?;
    context
        .audit_writer
        .upsert_live_order_pending(&result.lifecycle)
        .await?;
    context
        .audit_writer
        .mark_approval_consumed(&result.consumed_approval)
        .await?;
    record_live_tool_audit(
        context.audit_writer,
        LIVE_ORDER_SUBMIT_TOOL,
        ORDERS_LIVE_SUBMIT,
        &Ok(payload.clone()),
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

    let mut idempotency_store = IdempotencyStore::default();
    let result = match cancel_live_order(request, context.writer, &mut idempotency_store).await {
        Ok(result) => result,
        Err(error) => {
            handle_pending_order_error(
                context.audit_writer,
                &idempotency_key,
                &request_hash,
                &error,
            )
            .await?;
            record_live_tool_audit(
                context.audit_writer,
                LIVE_ORDER_CANCEL_TOOL,
                ORDERS_LIVE_CANCEL,
                &Err(error.clone()),
            )
            .await?;
            return Err(error);
        }
    };
    let payload = serde_json::to_value(&result.lifecycle).map_err(output_error)?;
    context
        .audit_writer
        .insert_order_idempotency(&idempotency_key, &request_hash, &payload)
        .await?;
    context
        .audit_writer
        .remove_live_order_pending(
            &result.lifecycle.account_id,
            &result.lifecycle.broker_order_id,
        )
        .await?;
    record_live_tool_audit(
        context.audit_writer,
        LIVE_ORDER_CANCEL_TOOL,
        ORDERS_LIVE_CANCEL,
        &Ok(payload.clone()),
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

async fn handle_pending_order_error(
    audit_writer: &SqliteAuditWriter,
    idempotency_key: &IdempotencyKey,
    request_hash: &str,
    error: &GatewayError,
) -> Result<(), GatewayError> {
    if is_writer_boundary_error(error.code) {
        audit_writer
            .mark_order_failed_after_writer(idempotency_key, request_hash, error)
            .await
    } else {
        audit_writer
            .delete_order_pending(idempotency_key, request_hash)
            .await
    }
}

const fn is_writer_boundary_error(code: ErrorCode) -> bool {
    matches!(
        code,
        ErrorCode::BrokerBackendUnavailable
            | ErrorCode::BrokerResponseInvalid
            | ErrorCode::BrokerSessionRequired
            | ErrorCode::OrderValidationFailed
    )
}

async fn record_live_tool_audit(
    audit_writer: &SqliteAuditWriter,
    tool_name: &str,
    scope: &str,
    result: &Result<Value, GatewayError>,
) -> Result<(), GatewayError> {
    let mut event = build_mcp_tool_event(
        tool_name,
        scope,
        if result.is_ok() {
            crate::internal::audit::AuditResultStatus::Completed
        } else {
            crate::internal::audit::AuditResultStatus::Refused
        },
    );
    if let Err(error) = result {
        event.error_code = Some(error.code);
    }
    audit_writer.append(&event).await
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
