//! Sidecar identity and pairing commands.

use crate::cli::output::print_output;
use crate::internal::domain::{ErrorCode, GatewayError, LocalUserId};
use crate::internal::sidecar::{SidecarId, SidecarIdentity, create_pairing, create_relay_session};
use time::OffsetDateTime;

/// Creates a sidecar identity record.
pub fn identity_create(
    display_name: Option<String>,
    public_key: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let identity = SidecarIdentity::new(public_key.to_string(), display_name)?;
    print_output(json, "sidecar identity created", &identity)
}

/// Creates a pairing record.
pub fn pairing_create(
    remote_instance_id: &str,
    sidecar_id: &str,
    user_id: &str,
    ttl_seconds: i64,
    json: bool,
) -> Result<(), GatewayError> {
    let Some(user_id) = LocalUserId::new(user_id) else {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            "User id is required for sidecar pairing",
            false,
            Some("Provide --user-id".to_string()),
        ));
    };
    let Some(sidecar_id) = SidecarId::from_string(sidecar_id) else {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            "Sidecar id is required for sidecar pairing",
            false,
            Some("Provide --sidecar-id".to_string()),
        ));
    };

    let pairing = create_pairing(remote_instance_id, sidecar_id, user_id, ttl_seconds);
    print_output(json, "sidecar pairing created", &pairing)
}

/// Revokes a pairing record by id.
pub fn pairing_revoke(pairing_id: &str, json: bool) -> Result<(), GatewayError> {
    if pairing_id.trim().is_empty() {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            "Pairing id is required",
            false,
            Some("Provide --pairing-id".to_string()),
        ));
    }
    print_output(
        json,
        "sidecar pairing revoked",
        &serde_json::json!({
            "pairing_id": pairing_id,
            "status": "revoked"
        }),
    )
}

/// Creates a relay session record.
pub fn session_create(
    remote_instance_id: &str,
    sidecar_id: &str,
    ttl_seconds: i64,
    json: bool,
) -> Result<(), GatewayError> {
    let sidecar_id = parse_sidecar_id(sidecar_id)?;
    let session = create_relay_session(sidecar_id, remote_instance_id, ttl_seconds);
    print_output(json, "sidecar relay session created", &session)
}

/// Validates a relay request and returns the sanitized forwarded request.
pub fn relay_accept(
    remote_instance_id: &str,
    sidecar_id: &str,
    ttl_seconds: i64,
    tool_name: &str,
    scope: &str,
    payload_json: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let sidecar_id = parse_sidecar_id(sidecar_id)?;
    let session = create_relay_session(sidecar_id, remote_instance_id, ttl_seconds);
    let payload = serde_json::from_str::<serde_json::Value>(payload_json).map_err(|_| {
        GatewayError::new(
            ErrorCode::ConfigInvalid,
            "Sidecar relay payload must be valid JSON",
            false,
            Some("Pass --payload-json with a JSON object".to_string()),
        )
    })?;
    let accepted = crate::internal::mcp::sidecar_relay::accept_sidecar_relay_request(
        Some(&session),
        tool_name,
        scope,
        &payload,
        OffsetDateTime::now_utc(),
    )?;
    print_output(json, "sidecar relay request accepted", &accepted)
}

fn parse_sidecar_id(sidecar_id: &str) -> Result<SidecarId, GatewayError> {
    SidecarId::from_string(sidecar_id).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::ConfigInvalid,
            "Sidecar id is required",
            false,
            Some("Provide --sidecar-id".to_string()),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{identity_create, relay_accept};
    use crate::internal::domain::ErrorCode;

    #[test]
    fn sidecar_identity_requires_explicit_public_key() -> Result<(), Box<dyn std::error::Error>> {
        let result = identity_create(None, "", true);
        let Err(error) = result else {
            return Err("missing sidecar public key must fail".into());
        };
        assert_eq!(error.code, ErrorCode::ConfigInvalid);
        Ok(())
    }

    #[test]
    fn sidecar_relay_accept_rejects_sensitive_payload() -> Result<(), Box<dyn std::error::Error>> {
        let result = relay_accept(
            "remote-1",
            "sidecar-1",
            60,
            "ibkr_accounts_list",
            "ibkr:accounts:read",
            r#"{"authorization":"Bearer secret"}"#,
            true,
        );
        let Err(error) = result else {
            return Err("sensitive sidecar payload must fail".into());
        };
        assert_eq!(error.code, ErrorCode::OutputUnsafe);
        Ok(())
    }
}
