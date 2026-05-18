//! MCP tool registry.

use super::{
    schemas::{ToolSchema, object_schema, safe_output_schema},
    tools::{
        order_preview::order_preview_schema,
        orders_live::{live_order_cancel_schema, live_order_submit_schema},
        orders_paper::{paper_order_cancel_schema, paper_order_submit_schema},
    },
};
use crate::internal::auth::{
    ACCOUNTS_READ, AUDIT_READ, HEALTH_READ, MARKETDATA_READ, ORDERS_READ, PORTFOLIO_READ,
    POSITIONS_READ, ScopeSet,
};
use crate::internal::domain::{ErrorCode, GatewayError};
use std::sync::OnceLock;

static BASE_BROKER_TOOL_SCHEMAS: OnceLock<Vec<ToolSchema>> = OnceLock::new();

/// Forbidden generic write-like MCP tool names.
pub const FORBIDDEN_TOOL_NAMES: &[&str] = &[
    "ibkr_order_intent_validate",
    "ibkr_order_preview_explain",
    "ibkr_order_submit",
    "ibkr_order_cancel",
    "ibkr_order_modify",
    "ibkr_order_approve",
];

/// Returns default local broker tool schemas.
#[must_use]
pub fn broker_tool_schemas() -> Vec<ToolSchema> {
    broker_tool_schemas_ref().to_vec()
}

/// Returns default local broker tool schemas without cloning.
#[must_use]
pub fn broker_tool_schemas_ref() -> &'static [ToolSchema] {
    base_broker_tool_schemas()
}

/// Returns local broker schemas with optional live trading tools.
#[must_use]
pub fn broker_tool_schemas_with_live(live_enabled: bool) -> Vec<ToolSchema> {
    let mut tools = broker_tool_schemas();
    if live_enabled {
        tools.push(live_order_submit_schema());
        tools.push(live_order_cancel_schema());
    }
    tools
}

/// Returns every explicit local MCP tool schema, including gated write tools.
#[must_use]
pub fn local_tool_schemas() -> Vec<ToolSchema> {
    let mut tools = broker_tool_schemas();
    tools.push(order_preview_schema());
    tools.push(paper_order_submit_schema());
    tools.push(paper_order_cancel_schema());
    tools.push(live_order_submit_schema());
    tools.push(live_order_cancel_schema());
    tools
}

/// Returns local MCP tool schemas visible to the provided scope set.
#[must_use]
pub fn local_tool_schemas_for_scopes(scopes: &ScopeSet) -> Vec<ToolSchema> {
    local_tool_schemas()
        .into_iter()
        .filter(|tool| scopes.contains(&tool.scope))
        .collect()
}

/// Returns the number of local broker tools without cloning schema values.
#[must_use]
pub fn broker_tool_schema_count() -> usize {
    broker_tool_schemas_ref().len()
}

/// Finds a local broker tool schema without rebuilding the registry.
#[must_use]
pub fn find_broker_tool_schema(name: &str) -> Option<&'static ToolSchema> {
    broker_tool_schemas_ref()
        .iter()
        .find(|tool| tool.name == name)
}

/// Finds any explicit local MCP tool schema.
#[must_use]
pub fn find_local_tool_schema(name: &str) -> Option<ToolSchema> {
    if let Some(tool) = find_broker_tool_schema(name) {
        return Some(tool.clone());
    }
    local_tool_schemas()
        .into_iter()
        .find(|tool| tool.name == name)
}

/// Finds a broker tool schema, optionally including live trading tools.
#[must_use]
pub fn find_broker_tool_schema_with_live(name: &str, live_enabled: bool) -> Option<ToolSchema> {
    if let Some(tool) = find_broker_tool_schema(name) {
        return Some(tool.clone());
    }
    if !live_enabled {
        return None;
    }
    broker_tool_schemas_with_live(true)
        .into_iter()
        .find(|tool| tool.name == name)
}

fn base_broker_tool_schemas() -> &'static [ToolSchema] {
    BASE_BROKER_TOOL_SCHEMAS.get_or_init(|| {
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
                "ibkr_order_status",
                ORDERS_READ,
                &["account_id", "broker_order_id"],
            ),
            tool("ibkr_executions_list", ORDERS_READ, &["account_id"]),
            tool("ibkr_audit_tail", AUDIT_READ, &["limit"]),
        ]
    })
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
        format!("MCP tool {name} is forbidden; use explicit preview, paper, or live-gated tools"),
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
