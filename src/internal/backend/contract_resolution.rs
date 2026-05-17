//! Shared contract candidate resolution rules.

use super::r#trait::BackendResult;
use crate::internal::domain::{ContractCandidate, ErrorCode, GatewayError};

/// Resolves exactly one unique contract candidate.
pub fn resolve_unique_contract(
    candidates: impl IntoIterator<Item = ContractCandidate>,
) -> BackendResult<ContractCandidate> {
    let mut unique = None;
    for candidate in candidates {
        if !candidate.is_unique_match {
            continue;
        }
        if unique.replace(candidate).is_some() {
            return Err(ambiguous_contract_error());
        }
    }
    unique.ok_or_else(ambiguous_contract_error)
}

fn ambiguous_contract_error() -> GatewayError {
    GatewayError::new(
        ErrorCode::InputAmbiguousContract,
        "Contract resolution is ambiguous",
        false,
        Some("Provide symbol, asset class, currency, and exchange".to_string()),
    )
}
