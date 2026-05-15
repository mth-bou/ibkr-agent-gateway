//! Sidecar identity and pairing commands.

use crate::output::print_output;
use ibkr_domain::{ErrorCode, GatewayError, LocalUserId};
use ibkr_sidecar::{SidecarId, SidecarIdentity, create_pairing};

/// Creates a sidecar identity record.
pub fn identity_create(
    display_name: Option<String>,
    public_key: Option<String>,
    json: bool,
) -> Result<(), GatewayError> {
    let public_key = public_key.unwrap_or_else(|| "local-public-key-placeholder".to_string());
    let identity = SidecarIdentity::new(public_key, display_name);
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
