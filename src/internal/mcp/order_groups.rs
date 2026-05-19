//! MCP handlers for bracket/OCA-style order groups.

use super::tools::order_groups::{
    BRACKET_ORDER_PREVIEW_TOOL, LIVE_BRACKET_ORDER_SUBMIT_TOOL, PAPER_BRACKET_ORDER_SUBMIT_TOOL,
};
use crate::cli::commands::account::parse_account_id;
use crate::internal::{
    approval::ApprovalId,
    audit::SqliteAuditWriter,
    auth::{ORDERS_LIVE_SUBMIT, ORDERS_PAPER_SUBMIT, ORDERS_PREVIEW, ScopeSet},
    backend::IbkrBackend,
    config::{LiveTradingConfig, PaperTradingConfig},
    domain::{
        AccountId, AccountMode, AssetClass, CurrencyCode, ErrorCode, GatewayError, LocalUserId,
        Money, OrderContractInput, OrderGroupId, OrderIntent, OrderIntentId, OrderSide,
        PreviewOrderType, Quantity, TimeInForce, ValidatedOrderGroup,
    },
    mcp::enforce_scope,
    orders::{
        IdempotencyKey, IdempotencyStore, KillSwitch, LiveGroupSubmitRequest,
        LocalCandidateLiveGroupWriter, LocalCandidatePaperGroupWriter, PaperGroupSubmitRequest,
        PaperToLiveMigrationChecklist, build_validated_order, create_bracket_order_preview,
        submit_live_group_order, submit_paper_group_order,
    },
    risk::{RiskDecision, RiskPolicy, validate_order_intent},
};
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;
use time::OffsetDateTime;

/// Server-owned dependencies for group order MCP handlers.
pub struct McpOrderGroupContext<'a> {
    /// Broker read backend used for contract resolution.
    pub backend: &'a dyn IbkrBackend,
    /// Durable audit and workflow state.
    pub audit_writer: &'a SqliteAuditWriter,
    /// Live config for live group submit.
    pub live_config: LiveTradingConfig,
    /// Kill switch for live group submit.
    pub kill_switch: KillSwitch,
    /// Migration acknowledgement.
    pub migration_checklist: PaperToLiveMigrationChecklist,
}

/// Dispatches one explicit group order MCP tool.
pub async fn handle_order_group_tool(
    context: &McpOrderGroupContext<'_>,
    scopes: &ScopeSet,
    tool_name: &str,
    args: &Value,
) -> Result<Value, GatewayError> {
    match tool_name {
        BRACKET_ORDER_PREVIEW_TOOL => handle_bracket_preview(context, scopes, args).await,
        PAPER_BRACKET_ORDER_SUBMIT_TOOL => handle_paper_bracket_submit(context, scopes, args).await,
        LIVE_BRACKET_ORDER_SUBMIT_TOOL => handle_live_bracket_submit(context, scopes, args).await,
        _ => Err(GatewayError::new(
            ErrorCode::ReadonlyWriteForbidden,
            format!("MCP tool {tool_name} is not an order group tool"),
            false,
            Some("Use a registered order group MCP tool".to_string()),
        )),
    }
}

/// Handles `ibkr_bracket_order_preview`.
pub async fn handle_bracket_preview(
    context: &McpOrderGroupContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_PREVIEW)?;
    let group = build_group_from_args(context.backend, args).await?;
    let preview = create_bracket_order_preview(&group)?;
    context
        .audit_writer
        .append_order_preview(&preview.parent, &group.parent)
        .await?;
    context
        .audit_writer
        .append_order_preview(&preview.take_profit, &group.take_profit)
        .await?;
    context
        .audit_writer
        .append_order_preview(&preview.stop_loss, &group.stop_loss)
        .await?;
    serde_json::to_value(preview).map_err(output_error)
}

/// Handles `ibkr_paper_bracket_order_submit`.
pub async fn handle_paper_bracket_submit(
    context: &McpOrderGroupContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_PAPER_SUBMIT)?;
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let group = load_group_from_approvals(context.audit_writer, &account_id, args).await?;
    let idempotency_key = IdempotencyKey::new(arg_string(args, "idempotency_key")?)?;
    let request = PaperGroupSubmitRequest {
        group,
        idempotency_key,
        paper_config: PaperTradingConfig {
            enabled: true,
            allowed_accounts: vec![account_id],
        },
    };
    let mut idempotency_store = IdempotencyStore::default();
    let lifecycle = submit_paper_group_order(
        request,
        &LocalCandidatePaperGroupWriter,
        &mut idempotency_store,
    )
    .await?;
    serde_json::to_value(lifecycle).map_err(output_error)
}

/// Handles `ibkr_live_bracket_order_submit`.
pub async fn handle_live_bracket_submit(
    context: &McpOrderGroupContext<'_>,
    scopes: &ScopeSet,
    args: &Value,
) -> Result<Value, GatewayError> {
    enforce_scope(scopes, ORDERS_LIVE_SUBMIT)?;
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let group = load_group_from_approvals(context.audit_writer, &account_id, args).await?;
    let idempotency_key = IdempotencyKey::new(arg_string(args, "idempotency_key")?)?;
    let request = LiveGroupSubmitRequest {
        group,
        idempotency_key,
        live_config: context.live_config.clone(),
        live_scope_granted: true,
        kill_switch: context.kill_switch.clone(),
        audit_available: true,
        migration_checklist: context.migration_checklist.clone(),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let lifecycle = submit_live_group_order(
        request,
        &LocalCandidateLiveGroupWriter,
        &mut idempotency_store,
    )
    .await?;
    serde_json::to_value(lifecycle).map_err(output_error)
}

async fn build_group_from_args(
    backend: &dyn IbkrBackend,
    args: &Value,
) -> Result<ValidatedOrderGroup, GatewayError> {
    let account_id = parse_account_id(arg_string(args, "account_id")?)?;
    let symbol = arg_string(args, "symbol")?;
    let side = parse_side(arg_string(args, "side")?)?;
    let exit_side = match side {
        OrderSide::Buy => OrderSide::Sell,
        OrderSide::Sell => OrderSide::Buy,
    };
    let quantity = parse_decimal(arg_string(args, "quantity")?, "quantity")?;
    let currency = parse_currency(
        args.get("currency")
            .and_then(Value::as_str)
            .unwrap_or("USD"),
    )?;
    let contract = backend.resolve_contract(symbol).await?;
    let group_id = OrderGroupId::new();
    let parent = build_leg(
        &account_id,
        symbol,
        side,
        quantity,
        PreviewOrderType::Limit,
        Some(parse_money(args, "entry_limit_price", &currency)?),
        None,
        &currency,
        &contract,
    )?;
    let take_profit = build_leg(
        &account_id,
        symbol,
        exit_side,
        quantity,
        PreviewOrderType::Limit,
        Some(parse_money(args, "take_profit_limit_price", &currency)?),
        None,
        &currency,
        &contract,
    )?;
    let stop_loss = build_leg(
        &account_id,
        symbol,
        exit_side,
        quantity,
        PreviewOrderType::Stop,
        None,
        Some(parse_money(args, "stop_loss_stop_price", &currency)?),
        &currency,
        &contract,
    )?;
    Ok(ValidatedOrderGroup {
        group_id,
        account_id,
        parent,
        take_profit,
        stop_loss,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_leg(
    account_id: &AccountId,
    symbol: &str,
    side: OrderSide,
    quantity: Decimal,
    order_type: PreviewOrderType,
    limit_price: Option<Money>,
    stop_price: Option<Money>,
    currency: &CurrencyCode,
    contract: &crate::internal::domain::ContractCandidate,
) -> Result<crate::internal::domain::ValidatedOrder, GatewayError> {
    let intent = OrderIntent {
        intent_id: OrderIntentId::new(),
        account_id: account_id.clone(),
        account_mode: AccountMode::Paper,
        contract: OrderContractInput::Query {
            symbol: symbol.to_string(),
            asset_class: AssetClass::Stock,
            currency: currency.clone(),
            exchange: Some("SMART".to_string()),
        },
        side,
        quantity: Quantity::new(quantity),
        order_type,
        limit_price,
        stop_price,
        trailing_amount: None,
        trailing_percent: None,
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
            "Bracket leg was refused by deterministic risk policy",
            false,
            Some("Inspect bracket leg risk refusal details".to_string()),
        ));
    };
    build_validated_order(
        &intent,
        contract,
        warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect(),
        300,
    )
}

async fn load_group_from_approvals(
    audit_writer: &SqliteAuditWriter,
    account_id: &AccountId,
    args: &Value,
) -> Result<ValidatedOrderGroup, GatewayError> {
    let parent = load_approved_order(audit_writer, account_id, "parent_approval_id", args).await?;
    let take_profit =
        load_approved_order(audit_writer, account_id, "take_profit_approval_id", args).await?;
    let stop_loss =
        load_approved_order(audit_writer, account_id, "stop_loss_approval_id", args).await?;
    Ok(ValidatedOrderGroup {
        group_id: OrderGroupId::new(),
        account_id: account_id.clone(),
        parent,
        take_profit,
        stop_loss,
    })
}

async fn load_approved_order(
    audit_writer: &SqliteAuditWriter,
    account_id: &AccountId,
    key: &str,
    args: &Value,
) -> Result<crate::internal::domain::ValidatedOrder, GatewayError> {
    let approval_id = ApprovalId::parse(arg_string(args, key)?)?;
    let approval = audit_writer
        .load_approval(&approval_id)
        .await?
        .ok_or_else(missing_approval)?;
    if &approval.account_id != account_id {
        return Err(GatewayError::new(
            ErrorCode::InputUnauthorizedAccount,
            "Approval account does not match requested group account",
            false,
            Some("Use approvals created for the same account".to_string()),
        ));
    }
    let preview = audit_writer
        .load_order_preview(&approval.preview_id)
        .await?
        .ok_or_else(missing_preview)?;
    Ok(preview.validated_order)
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
                Some("Send a valid MCP order group request".to_string()),
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

fn parse_money(args: &Value, key: &str, currency: &CurrencyCode) -> Result<Money, GatewayError> {
    Ok(Money {
        amount: parse_decimal(arg_string(args, key)?, key)?,
        currency: currency.clone(),
    })
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

fn missing_approval() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Bracket submit requires existing approval records",
        false,
        Some("Create approvals for all bracket previews".to_string()),
    )
}

fn missing_preview() -> GatewayError {
    GatewayError::new(
        ErrorCode::PaperApprovalRequired,
        "Bracket submit requires all approved previews to be present",
        false,
        Some("Create fresh bracket previews and approvals".to_string()),
    )
}

fn output_error(_error: serde_json::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::OutputUnsafe,
        "Failed to serialize MCP order group result",
        false,
        Some("Retry the MCP request".to_string()),
    )
}
