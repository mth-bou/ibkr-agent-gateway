//! Stable error codes and user-actionable error shape.

use super::identifiers::AuditEventId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Stable gateway error codes from the feature contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Invalid configuration.
    ConfigInvalid,
    /// Broker base URL is missing.
    ConfigMissingBrokerBaseUrl,
    /// TLS bypass is configured for a non-localhost URL.
    ConfigTlsBypassNonLocalhost,
    /// Write tools are forbidden by current configuration.
    ConfigWriteToolsForbidden,
    /// Remote MCP is forbidden by current configuration.
    ConfigRemoteMcpForbidden,
    /// Sidecar relay is forbidden by current configuration.
    ConfigSidecarForbidden,
    /// Live trading is forbidden by current configuration.
    ConfigLiveTradingForbidden,
    /// Required scope is missing.
    AuthMissingScope,
    /// Bearer token is missing.
    AuthTokenMissing,
    /// Bearer token is invalid.
    AuthTokenInvalid,
    /// Bearer token is expired.
    AuthTokenExpired,
    /// Bearer token issuer is invalid.
    AuthInvalidIssuer,
    /// Bearer token audience/resource is invalid.
    AuthInvalidAudience,
    /// An unknown or disallowed scope was requested.
    AuthScopeNotAllowedInMvp,
    /// Only local config auth is allowed in the current mode.
    AuthLocalOnlyMvp,
    /// Account context is missing.
    InputMissingAccount,
    /// Account is not authorized.
    InputUnauthorizedAccount,
    /// Account selection is ambiguous.
    InputAmbiguousAccount,
    /// Contract resolution is ambiguous.
    InputAmbiguousContract,
    /// Asset class is unsupported.
    InputUnsupportedAssetClass,
    /// Contract id is invalid.
    InputInvalidContract,
    /// Time range is invalid.
    InputInvalidTimeRange,
    /// Broker login is required.
    BrokerSessionRequired,
    /// Broker session expired.
    BrokerSessionExpired,
    /// Broker backend is unavailable.
    BrokerBackendUnavailable,
    /// Broker rate limit was reached.
    BrokerRateLimited,
    /// Broker capability is unavailable.
    BrokerCapabilityUnavailable,
    /// Broker response could not be mapped safely.
    BrokerResponseInvalid,
    /// Market data is unavailable.
    MarketDataUnavailable,
    /// Market data is delayed.
    MarketDataDelayed,
    /// Market data is stale.
    MarketDataStale,
    /// Market data is incomplete.
    MarketDataIncomplete,
    /// Historical bars are unavailable.
    HistoricalBarsUnavailable,
    /// A write-like request was refused by read-only policy.
    ReadonlyWriteForbidden,
    /// Generic order preview is forbidden outside the explicit preview flow.
    ReadonlyOrderPreviewForbidden,
    /// Generic order submit is forbidden outside explicit paper/live flows.
    ReadonlyOrderSubmitForbidden,
    /// Generic order cancel is forbidden outside explicit paper/live flows.
    ReadonlyOrderCancelForbidden,
    /// Output was refused because it may expose unsafe material.
    OutputUnsafe,
    /// Audit write failed.
    AuditWriteFailed,
    /// Audit read scope is missing.
    AuditReadForbidden,
    /// Order preview is disabled by local configuration.
    OrderPreviewDisabled,
    /// Order policy refused the intent.
    OrderPolicyRefused,
    /// Order intent validation failed.
    OrderValidationFailed,
    /// Paper trading is disabled.
    PaperTradingDisabled,
    /// Paper order approval is missing or invalid.
    PaperApprovalRequired,
    /// Idempotency key is missing or conflicts with a prior request.
    PaperIdempotencyConflict,
    /// Sidecar pairing is missing or invalid.
    SidecarPairingRequired,
    /// Sidecar relay session is unavailable.
    SidecarUnavailable,
    /// Sidecar relay session is invalid or expired.
    SidecarSessionInvalid,
    /// Live trading is disabled.
    LiveTradingDisabled,
    /// A required live trading gate is missing.
    LiveGateMissing,
    /// Live hard-limit policy refused the order.
    LiveLimitRefused,
    /// Live kill switch is closed.
    LiveKillSwitchClosed,
    /// Paper-to-live migration checklist is required.
    LiveMigrationRequired,
}

/// Structured error returned by CLI, MCP, and service layers.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema, Error)]
#[error("{code:?}: {message}")]
pub struct GatewayError {
    /// Stable error code.
    pub code: ErrorCode,
    /// User-actionable message that must not contain secrets.
    pub message: String,
    /// Whether retrying the same operation may succeed.
    pub retryable: bool,
    /// Optional safe user action.
    pub user_action: Option<String>,
    /// Optional audit event correlation.
    pub audit_event_id: Option<AuditEventId>,
}

impl GatewayError {
    /// Creates a new structured gateway error.
    #[must_use]
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        retryable: bool,
        user_action: Option<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            retryable,
            user_action,
            audit_event_id: None,
        }
    }

    /// Adds audit event correlation to the error.
    #[must_use]
    pub fn with_audit_event_id(mut self, audit_event_id: AuditEventId) -> Self {
        self.audit_event_id = Some(audit_event_id);
        self
    }
}
