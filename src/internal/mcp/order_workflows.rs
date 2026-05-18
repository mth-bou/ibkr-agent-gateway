//! MCP handlers for explicit preview and paper order tools.

use super::tools::{
    order_preview::ORDER_PREVIEW_TOOL,
    orders_paper::{PAPER_ORDER_CANCEL_TOOL, PAPER_ORDER_SUBMIT_TOOL},
};
use crate::cli::commands::account::parse_account_id;
use crate::internal::{
    approval::ApprovalId,
    audit::{
        OrderIdempotencyOperation, OrderIdempotencyRecoveryContext, OrderIdempotencyWorkflow,
        SqliteAuditWriter,
    },
    auth::{ORDERS_PAPER_CANCEL, ORDERS_PAPER_SUBMIT, ORDERS_PREVIEW, ScopeSet},
    backend::IbkrBackend,
    config::{OrderPreviewConfig, PaperTradingConfig},
    domain::{
        AccountMode, AssetClass, BrokerOrderId, CurrencyCode, ErrorCode, GatewayError, LocalUserId,
        Money, OrderContractInput, OrderIntent, OrderIntentId, OrderSide, PreviewOrderType,
        Quantity, TimeInForce,
    },
    mcp::enforce_scope,
    orders::{
        IdempotencyKey, IdempotencyStore, LocalCandidatePaperWriter, PaperCancelRequest,
        PaperSubmitRequest, build_validated_order, cancel_paper_order, create_order_preview,
        stable_request_hash, submit_paper_order,
    },
    risk::{RiskDecision, RiskPolicy, validate_order_intent},
};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use std::str::FromStr;
use time::OffsetDateTime;

/// Server-owned dependencies for preview and paper MCP order handlers.
pub struct McpOrderWorkflowContext<'a> {
    /// Broker read backend used for contract resolution.
    pub backend: &'a dyn IbkrBackend,
    /// Durable audit and workflow state.
    pub audit_writer: &'a SqliteAuditWriter,
}

/// Dispatches one explicit preview or paper MCP order tool.
pub async fn handle_order_workflow_tool(
    context: &McpOrderWorkflowContext<'_>,
    scopes: &ScopeSet,
    tool_name: &str,
    args: &Value,
) -> Result<Value, GatewayError> {
    match tool_name {
        ORDER_PREVIEW_TOOL => handle_order_preview(context, scopes, args).await,
        PAPER_ORDER_SUBMIT_TOOL => handle_paper_submit(context, scopes, args).await,
        PAPER_ORDER_CANCEL_TOOL => handle_paper_cancel(context, scopes, args).await,
        _ => Err(GatewayError::new(
            ErrorCode::ReadonlyWriteForbidden,
            format!("MCP tool {tool_name} is not a preview or paper order tool"),
            false,
            Some("Use a registered preview or paper MCP tool".to_string()),
        )),
    }
}

/// Handles `ibkr_order_preview`.
pub async fn handle_order_preview(
    context: &McpOrderWorkflowContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_PREVIEW)?;
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let symbol = arg_string(args, "symbol")?;
    let side = parse_side(arg_string(args, "side")?)?;
    let quantity = parse_decimal(arg_string(args, "quantity")?, "quantity")?;
    let limit_price = parse_decimal(arg_string(args, "limit_price")?, "limit price")?;
    ensure_literal(args, "order_type", "limit")?;
    ensure_literal(args, "time_in_force", "day")?;
    let currency = parse_currency(
        args.get("currency")
            .and_then(Value::as_str)
            .unwrap_or("USD"),
    )?;

    let config = OrderPreviewConfig {
        enabled: true,
        ..OrderPreviewConfig::default()
    };
    let intent = OrderIntent {
        intent_id: OrderIntentId::new(),
        account_id,
        account_mode: AccountMode::Paper,
        contract: OrderContractInput::Query {
            symbol: symbol.to_string(),
            asset_class: AssetClass::Stock,
            currency: currency.clone(),
            exchange: Some("SMART".to_string()),
        },
        side,
        quantity: Quantity::new(quantity),
        order_type: PreviewOrderType::Limit,
        limit_price: Some(Money {
            amount: limit_price,
            currency,
        }),
        time_in_force: TimeInForce::Day,
        rationale: None,
        created_by: LocalUserId::from_static("mcp-local-user"),
        created_at: OffsetDateTime::now_utc(),
    };

    let policy = RiskPolicy {
        enabled: true,
        ..RiskPolicy::default()
    };
    let decision = validate_order_intent(&intent, &policy)?;
    let RiskDecision::Allow { warnings } = decision else {
        return Err(GatewayError::new(
            ErrorCode::OrderPolicyRefused,
            "Order preview was refused by deterministic risk policy",
            false,
            Some("Inspect risk refusal details".to_string()),
        ));
    };

    let contract = context.backend.resolve_contract(symbol).await?;
    let validated = build_validated_order(
        &intent,
        contract.contract_id,
        warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect(),
        config.preview_expiration_seconds,
    )?;
    let preview = create_order_preview(
        &validated,
        crate::internal::domain::AuditEventId::new(),
        None,
        None,
    )?;
    context
        .audit_writer
        .append_order_preview(&preview, &validated)
        .await?;
    serde_json::to_value(preview).map_err(output_error)
}

/// Handles `ibkr_paper_order_submit`.
pub async fn handle_paper_submit(
    context: &McpOrderWorkflowContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_PAPER_SUBMIT)?;
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let approval_id = ApprovalId::parse(arg_string(args, "approval_id")?)?;
    let approval = context
        .audit_writer
        .load_approval(&approval_id)
        .await?
        .ok_or_else(missing_approval)?;
    let preview_record = context
        .audit_writer
        .load_order_preview(&approval.preview_id)
        .await?
        .ok_or_else(missing_preview)?;
    let idempotency_key = IdempotencyKey::new(arg_string(args, "idempotency_key")?)?;
    let request_hash = stable_request_hash(
        "mcp.paper.submit",
        &PaperSubmitMcpFingerprint {
            account_id: account_id.as_str(),
            approval_id: approval_id.as_uuid().to_string(),
        },
    )?;
    if let Some(payload) = context
        .audit_writer
        .replay_order_idempotency(&idempotency_key, &request_hash)
        .await?
    {
        return Ok(payload);
    }

    let request = PaperSubmitRequest {
        order: preview_record.validated_order,
        approval,
        idempotency_key: idempotency_key.clone(),
        paper_config: PaperTradingConfig {
            enabled: true,
            allowed_accounts: vec![account_id],
        },
    };
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Paper,
        operation: OrderIdempotencyOperation::Submit,
        account_id: request.order.account_id.clone(),
        broker_order_id: None,
    };
    context
        .audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;

    let mut idempotency_store = IdempotencyStore::default();
    let result =
        match submit_paper_order(request, &LocalCandidatePaperWriter, &mut idempotency_store).await
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
        .insert_order_idempotency(&idempotency_key, &request_hash, &payload)
        .await?;
    context
        .audit_writer
        .mark_approval_consumed(&result.consumed_approval)
        .await?;
    Ok(payload)
}

/// Handles `ibkr_paper_order_cancel`.
pub async fn handle_paper_cancel(
    context: &McpOrderWorkflowContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_PAPER_CANCEL)?;
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let broker_order_id = parse_broker_order_id(arg_string(args, "broker_order_id")?)?;
    let idempotency_key = IdempotencyKey::new(arg_string(args, "idempotency_key")?)?;
    let request_hash = stable_request_hash(
        "mcp.paper.cancel",
        &PaperCancelMcpFingerprint {
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

    let request = PaperCancelRequest {
        account_id: account_id.clone(),
        broker_order_id: broker_order_id.clone(),
        idempotency_key: idempotency_key.clone(),
        paper_config: PaperTradingConfig {
            enabled: true,
            allowed_accounts: vec![account_id],
        },
    };
    let recovery_context = OrderIdempotencyRecoveryContext {
        workflow: OrderIdempotencyWorkflow::Paper,
        operation: OrderIdempotencyOperation::Cancel,
        account_id: request.account_id.clone(),
        broker_order_id: Some(broker_order_id),
    };
    context
        .audit_writer
        .insert_order_pending_with_context(&idempotency_key, &request_hash, Some(&recovery_context))
        .await?;

    let mut idempotency_store = IdempotencyStore::default();
    let result =
        match cancel_paper_order(request, &LocalCandidatePaperWriter, &mut idempotency_store).await
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
        .insert_order_idempotency(&idempotency_key, &request_hash, &payload)
        .await?;
    Ok(payload)
}

#[derive(Serialize)]
struct PaperSubmitMcpFingerprint<'a> {
    account_id: &'a str,
    approval_id: String,
}

#[derive(Serialize)]
struct PaperCancelMcpFingerprint<'a> {
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

fn arg_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, GatewayError> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            GatewayError::new(
                ErrorCode::ConfigInvalid,
                format!("Missing MCP argument: {key}"),
                false,
                Some("Send a valid MCP tools/call request".to_string()),
            )
        })
}

fn parse_decimal(value: &str, label: &str) -> Result<Decimal, GatewayError> {
    Decimal::from_str(value).map_err(|_| {
        GatewayError::new(
            ErrorCode::OrderValidationFailed,
            format!("Invalid decimal {label}"),
            false,
            Some(format!("Provide a valid decimal {label}")),
        )
    })
}

fn parse_side(value: &str) -> Result<OrderSide, GatewayError> {
    match value {
        "buy" => Ok(OrderSide::Buy),
        "sell" => Ok(OrderSide::Sell),
        _ => Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Side must be buy or sell",
            false,
            Some("Use side buy or sell".to_string()),
        )),
    }
}

fn parse_currency(value: &str) -> Result<CurrencyCode, GatewayError> {
    CurrencyCode::new(value).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Currency must be a three-letter code",
            false,
            Some("Use a valid ISO currency code".to_string()),
        )
    })
}

fn parse_broker_order_id(value: &str) -> Result<BrokerOrderId, GatewayError> {
    BrokerOrderId::new(value).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Broker order id is required",
            false,
            Some("Provide a broker order id".to_string()),
        )
    })
}

fn ensure_literal(args: &Value, key: &str, expected: &str) -> Result<(), GatewayError> {
    let value = arg_string(args, key)?;
    if value.eq_ignore_ascii_case(expected) {
        return Ok(());
    }
    Err(GatewayError::new(
        ErrorCode::OrderValidationFailed,
        format!("{key} must be {expected}"),
        false,
        Some(format!("Use {key}: {expected}")),
    ))
}

fn missing_approval() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Paper submit requires an existing approval record",
        false,
        Some("Create an approval and pass its approval_id".to_string()),
    )
}

fn missing_preview() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Paper submit requires the approved preview to be present",
        false,
        Some("Create a fresh preview and approval before paper submit".to_string()),
    )
}

fn output_error(_error: serde_json::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::OutputUnsafe,
        "Failed to serialize MCP order workflow result",
        false,
        Some("Retry the MCP request".to_string()),
    )
}
