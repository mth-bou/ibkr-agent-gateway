//! Read-only MCP tool registry.

use crate::{
    schemas::{ToolSchema, object_schema, safe_output_schema},
    tools::orders_live::{live_order_cancel_schema, live_order_submit_schema},
};
use ibkr_auth::{
    ACCOUNTS_READ, AUDIT_READ, HEALTH_READ, MARKETDATA_READ, ORDERS_PAPER_CANCEL,
    ORDERS_PAPER_SUBMIT, ORDERS_PREVIEW, ORDERS_READ, PORTFOLIO_READ, POSITIONS_READ,
};
use ibkr_domain::{ErrorCode, GatewayError};

/// Forbidden write-like MCP tool names in the read-only MVP.
pub const FORBIDDEN_TOOL_NAMES: &[&str] = &[
    "ibkr_order_intent_validate",
    "ibkr_order_preview_explain",
    "ibkr_order_submit",
    "ibkr_order_cancel",
    "ibkr_order_modify",
    "ibkr_order_approve",
];

/// Returns local broker read-only tool schemas.
#[must_use]
pub fn broker_tool_schemas() -> Vec<ToolSchema> {
    base_broker_tool_schemas()
}

/// Returns local broker schemas with optional live trading tools.
#[must_use]
pub fn broker_tool_schemas_with_live(live_enabled: bool) -> Vec<ToolSchema> {
    let mut tools = base_broker_tool_schemas();
    if live_enabled {
        tools.push(live_order_submit_schema());
        tools.push(live_order_cancel_schema());
    }
    tools
}

fn base_broker_tool_schemas() -> Vec<ToolSchema> {
    vec![
        tool("ibkr_health", HEALTH_READ, &[]),
        tool("ibkr_backend_status", HEALTH_READ, &[]),
        tool("ibkr_session_requirements", HEALTH_READ, &[]),
        tool("ibkr_accounts_list", ACCOUNTS_READ, &[]),
        tool("ibkr_account_summary", PORTFOLIO_READ, &["account_id"]),
        tool("ibkr_positions_list", POSITIONS_READ, &["account_id"]),
        tool("ibkr_portfolio_snapshot", PORTFOLIO_READ, &["account_id"]),
        tool("ibkr_contracts_search", MARKETDATA_READ, &["query"]),
        tool("ibkr_contract_resolve", MARKETDATA_READ, &["symbol"]),
        tool("ibkr_market_snapshot", MARKETDATA_READ, &["contract_id"]),
        tool(
            "ibkr_historical_bars",
            MARKETDATA_READ,
            &["contract_id", "duration", "bar_size"],
        ),
        tool("ibkr_orders_list", ORDERS_READ, &["account_id"]),
        tool(
            "ibkr_order_preview",
            ORDERS_PREVIEW,
            &[
                "account_id",
                "symbol",
                "side",
                "quantity",
                "order_type",
                "limit_price",
                "time_in_force",
            ],
        ),
        tool(
            "ibkr_order_status",
            ORDERS_READ,
            &["account_id", "broker_order_id"],
        ),
        tool("ibkr_executions_list", ORDERS_READ, &["account_id"]),
        tool("ibkr_audit_tail", AUDIT_READ, &["limit"]),
        tool(
            "ibkr_paper_order_submit",
            ORDERS_PAPER_SUBMIT,
            &["account_id", "approval_id", "idempotency_key"],
        ),
        tool(
            "ibkr_paper_order_cancel",
            ORDERS_PAPER_CANCEL,
            &["account_id", "broker_order_id", "idempotency_key"],
        ),
    ]
}

/// Returns true when the tool is forbidden in this phase.
#[must_use]
pub fn is_forbidden_tool_name(name: &str) -> bool {
    FORBIDDEN_TOOL_NAMES.contains(&name)
}

/// Refuses a forbidden write-like tool call.
pub fn refuse_forbidden_tool(name: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::ReadonlyWriteForbidden,
        format!("MCP tool {name} is forbidden in the read-only MVP"),
        false,
        Some("Use a later feature spec for preview or trading".to_string()),
    )
}

/// Audit tool scope.
#[must_use]
pub const fn audit_scope() -> &'static str {
    AUDIT_READ
}

fn tool(name: &str, scope: &str, required: &[&str]) -> ToolSchema {
    ToolSchema {
        name: name.to_string(),
        scope: scope.to_string(),
        input_schema: object_schema(required),
        output_schema: safe_output_schema(),
    }
}
