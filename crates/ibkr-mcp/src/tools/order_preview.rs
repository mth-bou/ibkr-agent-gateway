//! Order preview MCP tool.

use crate::schemas::{ToolSchema, order_preview_input_schema, safe_output_schema};
use ibkr_auth::ORDERS_PREVIEW;

/// Preview tool.
pub const ORDER_PREVIEW_TOOL: &str = "ibkr_order_preview";

/// Schema for the order preview tool.
#[must_use]
pub fn order_preview_schema() -> ToolSchema {
    ToolSchema {
        name: ORDER_PREVIEW_TOOL.to_string(),
        scope: ORDERS_PREVIEW.to_string(),
        input_schema: order_preview_input_schema(),
        output_schema: safe_output_schema(),
    }
}
