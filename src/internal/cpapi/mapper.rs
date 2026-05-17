//! Maps Client Portal Gateway models into domain models.

use super::models::{
    CpapiAccount, CpapiContractCandidate, CpapiSessionResponse, CpapiTickleResponse,
};
use crate::internal::audit::AuditHmacKey;
use crate::internal::domain::{
    AccountId, AccountMode, AssetClass, BrokerAccount, BrokerBackendKind, BrokerSessionStatus,
    BrokerSessionVisibility, ContractCandidate, ContractId, CurrencyCode, ErrorCode, GatewayError,
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
///
/// The `hmac_key` is required to derive the audit-safe `account_id_hash` so the
/// raw broker account identifier never reaches persistent storage in clear form.
pub fn map_account(
    account: CpapiAccount,
    hmac_key: &AuditHmacKey,
) -> Result<BrokerAccount, GatewayError> {
    let account_id = AccountId::new(account.account_id).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::BrokerResponseInvalid,
            "Broker account id was missing",
            true,
            Some("Retry account discovery".to_string()),
        )
    })?;
    let account_id_hash = hmac_key.compute_account_id_hash(account_id.as_str())?;
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

/// Maps one CPAPI contract candidate.
pub fn map_contract_candidate(
    candidate: CpapiContractCandidate,
) -> Result<ContractCandidate, GatewayError> {
    let asset_class = match candidate.asset_class.as_str() {
        "stock" => AssetClass::Stock,
        "etf" => AssetClass::Etf,
        _ => {
            return Err(GatewayError::new(
                ErrorCode::InputUnsupportedAssetClass,
                "Unsupported asset class",
                false,
                Some("Use stock or ETF".to_string()),
            ));
        }
    };
    let contract_id = ContractId::new(candidate.contract_id).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::InputInvalidContract,
            "Contract id was missing",
            false,
            Some("Use a resolved contract id".to_string()),
        )
    })?;
    let currency = CurrencyCode::new(candidate.currency).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::MarketDataIncomplete,
            "Contract currency was missing or invalid",
            true,
            Some("Retry contract lookup with currency".to_string()),
        )
    })?;

    Ok(ContractCandidate {
        contract_id,
        symbol: candidate.symbol,
        description: candidate.description,
        asset_class,
        exchange: candidate.exchange,
        currency,
        is_unique_match: candidate.is_unique_match,
    })
}
