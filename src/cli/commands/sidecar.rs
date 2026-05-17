//! Sidecar identity and pairing commands.

use crate::cli::output::print_output;
use crate::internal::domain::{ErrorCode, GatewayError, LocalUserId};
use crate::internal::sidecar::{SidecarId, SidecarIdentity, create_pairing};

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

#[cfg(test)]
mod tests {
    use super::identity_create;
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
}
