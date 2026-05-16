//! Configuration validation helpers.

use crate::internal::domain::{ErrorCode, GatewayError};
use url::Url;

/// Validates that TLS bypass is only accepted for localhost CP Gateway URLs.
pub fn validate_tls_bypass_localhost_only(
    base_url: &Url,
    verify_tls: bool,
) -> Result<(), GatewayError> {
    if verify_tls {
        return Ok(());
    }

    let host = base_url.host_str().unwrap_or_default();
    if matches!(host, "localhost" | "127.0.0.1" | "::1") {
        Ok(())
    } else {
        Err(GatewayError::new(
            ErrorCode::ConfigTlsBypassNonLocalhost,
            "TLS verification can only be disabled for localhost Client Portal Gateway URLs",
            false,
            Some("Enable TLS verification or use a localhost broker base URL".to_string()),
        ))
    }
}
