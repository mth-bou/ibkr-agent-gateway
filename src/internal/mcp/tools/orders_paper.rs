//! Paper order MCP tools.

use super::super::schemas::{ToolSchema, object_schema, safe_output_schema};
use crate::internal::auth::{ORDERS_PAPER_CANCEL, ORDERS_PAPER_SUBMIT};

/// Paper submit tool.
pub const PAPER_ORDER_SUBMIT_TOOL: &str = "ibkr_paper_order_submit";
/// Paper cancel tool.
pub const PAPER_ORDER_CANCEL_TOOL: &str = "ibkr_paper_order_cancel";

/// Schema for paper submit.
#[must_use]
pub fn paper_order_submit_schema() -> ToolSchema {
    ToolSchema {
        name: PAPER_ORDER_SUBMIT_TOOL.to_string(),
        scope: ORDERS_PAPER_SUBMIT.to_string(),
        input_schema: object_schema(&["account_id", "approval_id", "idempotency_key"]),
        output_schema: safe_output_schema(),
    }
}

/// Schema for paper cancel.
#[must_use]
pub fn paper_order_cancel_schema() -> ToolSchema {
    ToolSchema {
        name: PAPER_ORDER_CANCEL_TOOL.to_string(),
        scope: ORDERS_PAPER_CANCEL.to_string(),
        input_schema: object_schema(&["account_id", "broker_order_id", "idempotency_key"]),
        output_schema: safe_output_schema(),
    }
}
