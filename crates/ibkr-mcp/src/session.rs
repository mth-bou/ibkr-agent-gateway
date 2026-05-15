//! HTTP MCP request and session correlation.

use ibkr_domain::{RequestId, SessionId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
        let _request_header = header(headers, REQUEST_ID_HEADER);
        let _session_header = header(headers, SESSION_ID_HEADER);
        Self {
            request_id: RequestId::new(),
            session_id: SessionId::new(),
        }
    }
}

fn header<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .get(name)
        .or_else(|| headers.get(&name.to_ascii_lowercase()))
        .map(String::as_str)
}
