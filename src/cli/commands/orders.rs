//! Read-only order and execution commands.

use crate::cli::{
    audit::build_cli_audit_event, commands::account::parse_account_id, output::print_output,
};
use crate::internal::audit::{AuditEventType, AuditResultStatus};
use crate::internal::auth::ORDERS_READ;
use crate::internal::backend::IbkrBackend;
use crate::internal::domain::{ErrorCode, GatewayError};

/// Runs `ibkr-agent orders list`.
pub async fn list(
    backend: &dyn IbkrBackend,
    account: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let value = backend.orders(&account_id).await?;
    let _event = build_cli_audit_event(
        "ibkr_orders_list",
        ORDERS_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "orders returned", &value)
}

/// Runs `ibkr-agent orders status`.
pub async fn status(
    backend: &dyn IbkrBackend,
    account: &str,
    broker_order_id: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let value = backend.order_status(&account_id, broker_order_id).await?;
    let _event = build_cli_audit_event(
        "ibkr_order_status",
        ORDERS_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "order status returned", &value)
}

/// Runs `ibkr-agent executions list`.
pub async fn executions(
    backend: &dyn IbkrBackend,
    account: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let value = backend.executions(&account_id).await?;
    let _event = build_cli_audit_event(
        "ibkr_executions_list",
        ORDERS_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "executions returned", &value)
}

/// Refuses generic write-like order commands that have explicit safer flows.
pub fn refuse_write(action: &str) -> Result<(), GatewayError> {
    Err(GatewayError::new(
        ErrorCode::ReadonlyWriteForbidden,
        format!("Order {action} is forbidden; use explicit preview, paper, or live-gated flows"),
        false,
        Some("Use explicit preview, paper, or live-gated commands".to_string()),
    ))
}
