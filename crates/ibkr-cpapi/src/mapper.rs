//! Maps Client Portal Gateway models into domain models.

use crate::models::{CpapiAccount, CpapiSessionResponse, CpapiTickleResponse};
use ibkr_domain::{
    AccountId, AccountIdHash, AccountMode, BrokerAccount, BrokerBackendKind, BrokerSessionStatus,
    BrokerSessionVisibility, CurrencyCode, ErrorCode, GatewayError,
};
use time::OffsetDateTime;

/// Maps a CPAPI session response into a safe domain status.
#[must_use]
pub fn map_session_response(response: CpapiSessionResponse) -> BrokerSessionStatus {
    let (status, error_code, user_action) = if response.authenticated {
        (BrokerSessionVisibility::Usable, None, None)
    } else {
        (
            BrokerSessionVisibility::ManualActionRequired,
            Some(ErrorCode::BrokerSessionRequired),
            Some(
                response
                    .message
                    .unwrap_or_else(|| "Complete broker login manually".to_string()),
            ),
        )
    };

    BrokerSessionStatus {
        status,
        backend: BrokerBackendKind::ClientPortalGateway,
        checked_at: OffsetDateTime::now_utc(),
        last_keepalive_at: None,
        user_action,
        error_code,
    }
}

/// Maps a CPAPI keepalive response into a safe domain status.
#[must_use]
pub fn map_tickle_response(response: CpapiTickleResponse) -> BrokerSessionStatus {
    let now = OffsetDateTime::now_utc();
    if response.ok {
        BrokerSessionStatus {
            status: BrokerSessionVisibility::Usable,
            backend: BrokerBackendKind::ClientPortalGateway,
            checked_at: now,
            last_keepalive_at: Some(now),
            user_action: None,
            error_code: None,
        }
    } else {
        BrokerSessionStatus {
            status: BrokerSessionVisibility::ManualActionRequired,
            backend: BrokerBackendKind::ClientPortalGateway,
            checked_at: now,
            last_keepalive_at: None,
            user_action: response
                .message
                .or_else(|| Some("Reauthenticate broker session".to_string())),
            error_code: Some(ErrorCode::BrokerSessionExpired),
        }
    }
}

/// Maps one CPAPI account into safe metadata.
pub fn map_account(account: CpapiAccount) -> Result<BrokerAccount, GatewayError> {
    let account_id = AccountId::new(account.account_id).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::BrokerResponseInvalid,
            "Broker account id was missing",
            true,
            Some("Retry account discovery".to_string()),
        )
    })?;
    let account_id_hash = AccountIdHash::new(format!("fixture-hmac:{}", account_id.as_str()))
        .ok_or_else(|| {
            GatewayError::new(
                ErrorCode::BrokerResponseInvalid,
                "Could not create account audit hash",
                true,
                Some("Retry account discovery".to_string()),
            )
        })?;
    let account_mode = match account.account_mode.as_deref() {
        Some("paper") => AccountMode::Paper,
        Some("live") => AccountMode::Live,
        _ => AccountMode::Unknown,
    };
    let base_currency = account.base_currency.and_then(CurrencyCode::new);

    Ok(BrokerAccount {
        account_id,
        account_id_hash,
        account_label: account.account_label,
        account_mode,
        base_currency,
        metadata_redacted: true,
    })
}
