//! Account context validation helpers.

use crate::internal::domain::{AccountId, ErrorCode, GatewayError};

/// Requires an explicit account id for account-scoped reads.
pub fn require_account_id(account_id: Option<AccountId>) -> Result<AccountId, GatewayError> {
    account_id.ok_or_else(|| {
        GatewayError::new(
            ErrorCode::InputMissingAccount,
            "Account context is required",
            false,
            Some("Select one account explicitly".to_string()),
        )
    })
}
