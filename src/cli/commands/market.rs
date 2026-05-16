//! Market data commands.

use crate::cli::{audit::build_cli_audit_event, output::print_output};
use crate::internal::audit::{AuditEventType, AuditResultStatus};
use crate::internal::auth::MARKETDATA_READ;
use crate::internal::backend::IbkrBackend;
use crate::internal::domain::{ContractId, ErrorCode, GatewayError, HistoricalBarsRequest};

/// Runs `ibkr-agent market snapshot`.
pub async fn snapshot(
    backend: &dyn IbkrBackend,
    contract_id: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let contract_id = parse_contract_id(contract_id)?;
    let value = backend.market_snapshot(&contract_id).await?;
    let _event = build_cli_audit_event(
        "ibkr_market_snapshot",
        MARKETDATA_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "market snapshot returned", &value)
}

/// Runs `ibkr-agent market bars`.
pub async fn bars(
    backend: &dyn IbkrBackend,
    contract_id: &str,
    duration: &str,
    bar_size: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let contract_id = parse_contract_id(contract_id)?;
    let request = HistoricalBarsRequest {
        contract_id,
        duration: duration.to_string(),
        bar_size: bar_size.to_string(),
        outside_regular_trading_hours: false,
    };
    let value = backend.historical_bars(&request).await?;
    let _event = build_cli_audit_event(
        "ibkr_historical_bars",
        MARKETDATA_READ,
        AuditEventType::ToolCompleted,
        AuditResultStatus::Completed,
    );
    print_output(json, "historical bars returned", &value)
}

fn parse_contract_id(contract_id: &str) -> Result<ContractId, GatewayError> {
    ContractId::new(contract_id).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::InputInvalidContract,
            "Contract id is required",
            false,
            Some("Use a resolved contract id".to_string()),
        )
    })
}
