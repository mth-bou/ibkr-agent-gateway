//! HTTP MCP request and session correlation.

use crate::internal::domain::{RequestId, SessionId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Header carrying the remote MCP request id.
pub const REQUEST_ID_HEADER: &str = "x-request-id";
/// Header carrying the remote MCP session id.
pub const SESSION_ID_HEADER: &str = "mcp-session-id";

/// Request/session identifiers propagated through remote MCP.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HttpMcpSessionIds {
    /// Request correlation id.
    pub request_id: RequestId,
    /// Session correlation id.
    pub session_id: SessionId,
}

impl HttpMcpSessionIds {
    /// Builds fresh ids when remote headers are absent or malformed.
    #[must_use]
    pub fn from_headers(headers: &BTreeMap<String, String>) -> Self {
        Self {
            request_id: header_uuid(headers, REQUEST_ID_HEADER)
                .map(RequestId::from_uuid)
                .unwrap_or_default(),
            session_id: header_uuid(headers, SESSION_ID_HEADER)
                .map(SessionId::from_uuid)
                .unwrap_or_default(),
        }
    }
}

fn header<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

fn header_uuid(headers: &BTreeMap<String, String>, name: &str) -> Option<Uuid> {
    header(headers, name).and_then(|value| Uuid::parse_str(value).ok())
}
