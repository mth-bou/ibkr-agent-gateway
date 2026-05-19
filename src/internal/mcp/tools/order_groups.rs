//! Group order MCP tools.

use super::super::schemas::{ToolSchema, object_schema, safe_output_schema};
use crate::internal::auth::{ORDERS_LIVE_SUBMIT, ORDERS_PAPER_SUBMIT, ORDERS_PREVIEW};

/// Bracket preview tool.
pub const BRACKET_ORDER_PREVIEW_TOOL: &str = "ibkr_bracket_order_preview";
/// Paper bracket submit tool.
pub const PAPER_BRACKET_ORDER_SUBMIT_TOOL: &str = "ibkr_paper_bracket_order_submit";
/// Live bracket submit tool.
pub const LIVE_BRACKET_ORDER_SUBMIT_TOOL: &str = "ibkr_live_bracket_order_submit";

/// Schema for bracket preview.
#[must_use]
pub fn bracket_order_preview_schema() -> ToolSchema {
    ToolSchema {
        name: BRACKET_ORDER_PREVIEW_TOOL.to_string(),
        scope: ORDERS_PREVIEW.to_string(),
        input_schema: object_schema(&[
            "account_id",
            "symbol",
            "side",
            "quantity",
            "entry_limit_price",
            "take_profit_limit_price",
            "stop_loss_stop_price",
        ]),
        output_schema: safe_output_schema(),
    }
}

/// Schema for paper bracket submit.
#[must_use]
pub fn paper_bracket_order_submit_schema() -> ToolSchema {
    ToolSchema {
        name: PAPER_BRACKET_ORDER_SUBMIT_TOOL.to_string(),
        scope: ORDERS_PAPER_SUBMIT.to_string(),
        input_schema: object_schema(&[
            "account_id",
            "parent_approval_id",
            "take_profit_approval_id",
            "stop_loss_approval_id",
            "idempotency_key",
        ]),
        output_schema: safe_output_schema(),
    }
}

/// Schema for live bracket submit.
#[must_use]
pub fn live_bracket_order_submit_schema() -> ToolSchema {
    ToolSchema {
        name: LIVE_BRACKET_ORDER_SUBMIT_TOOL.to_string(),
        scope: ORDERS_LIVE_SUBMIT.to_string(),
        input_schema: object_schema(&[
            "account_id",
            "parent_approval_id",
            "take_profit_approval_id",
            "stop_loss_approval_id",
            "idempotency_key",
        ]),
        output_schema: safe_output_schema(),
    }
}
