//! Local Client Portal Gateway forwarding boundary.

use super::ForwardedBrokerRequest;
use crate::internal::domain::{ErrorCode, GatewayError, RequestId};
use crate::internal::encoding::sha256_hex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Safe response returned to the remote gateway after local forwarding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ForwardedBrokerResponse {
    /// Request id.
    pub request_id: RequestId,
    /// Whether the local CP Gateway session was usable.
    pub local_session_usable: bool,
    /// Safe user action when manual IBKR login is required.
    pub manual_action: Option<String>,
}

/// Builds a sanitized forwarded broker request.
pub fn build_forwarded_broker_request(
    tool_name: impl Into<String>,
    scope: impl Into<String>,
    payload: &serde_json::Value,
) -> Result<ForwardedBrokerRequest, GatewayError> {
    assert_safe_payload(payload)?;
    Ok(ForwardedBrokerRequest {
        request_id: RequestId::new(),
        tool_name: tool_name.into(),
        scope: scope.into(),
        payload_hash: sha256_hex(payload.to_string().as_bytes()),
        created_at: OffsetDateTime::now_utc(),
    })
}

fn assert_safe_payload(value: &serde_json::Value) -> Result<(), GatewayError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                let lowered = key.to_ascii_lowercase();
                if lowered.contains("cookie")
                    || lowered.contains("authorization")
                    || lowered.contains("token")
                    || lowered.contains("secret")
                    || lowered.contains("credential")
                    || lowered.contains("path")
                {
                    return Err(GatewayError::new(
                        ErrorCode::OutputUnsafe,
                        "Forwarded broker request contains sensitive material",
                        false,
                        Some(
                            "Remove cookies, credentials, tokens, headers, and local paths"
                                .to_string(),
                        ),
                    ));
                }
                assert_safe_payload(nested)?;
            }
            Ok(())
        }
        serde_json::Value::Array(values) => {
            for nested in values {
                assert_safe_payload(nested)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
