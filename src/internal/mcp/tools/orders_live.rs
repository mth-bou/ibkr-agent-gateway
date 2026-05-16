//! Live order MCP tools.

use super::super::schemas::{ToolSchema, object_schema, safe_output_schema};
use crate::internal::auth::{ORDERS_LIVE_CANCEL, ORDERS_LIVE_SUBMIT};

/// Live submit tool.
pub const LIVE_ORDER_SUBMIT_TOOL: &str = "ibkr_live_order_submit";
/// Live cancel tool.
pub const LIVE_ORDER_CANCEL_TOOL: &str = "ibkr_live_order_cancel";

/// Schema for live submit.
#[must_use]
pub fn live_order_submit_schema() -> ToolSchema {
    ToolSchema {
        name: LIVE_ORDER_SUBMIT_TOOL.to_string(),
        scope: ORDERS_LIVE_SUBMIT.to_string(),
        input_schema: object_schema(&["account_id", "approval_id", "idempotency_key"]),
        output_schema: safe_output_schema(),
    }
}

/// Schema for live cancel.
#[must_use]
pub fn live_order_cancel_schema() -> ToolSchema {
    ToolSchema {
        name: LIVE_ORDER_CANCEL_TOOL.to_string(),
        scope: ORDERS_LIVE_CANCEL.to_string(),
        input_schema: object_schema(&["account_id", "broker_order_id", "idempotency_key"]),
        output_schema: safe_output_schema(),
    }
}
