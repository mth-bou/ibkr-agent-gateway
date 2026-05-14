# Contract: Audit Events

Audit events are append-only records for significant gateway behavior. They must
be written for both allowed and denied operations.

## Required Event Types

- `tool.called`
- `tool.denied_scope`
- `tool.completed`
- `tool.failed`
- `backend.session.changed`

## Event Shape

```json
{
  "event_id": "018f-example",
  "event_type": "tool.completed",
  "timestamp": "2026-05-14T10:15:30Z",
  "user_id": "local-user",
  "session_id": "session-example",
  "request_id": "req-example",
  "account_id_hash": "sha256:example",
  "tool_name": "ibkr_accounts_list",
  "scopes": ["ibkr:accounts:read"],
  "decision": "allow",
  "error_code": null,
  "input_hash": "sha256:example",
  "output_hash": "sha256:example",
  "redactions": ["account_id_hash"],
  "metadata": {
    "backend": "client_portal_gateway",
    "status": "completed"
  }
}
```

## Redaction Rules

Audit events must never store raw:

- OAuth or bearer tokens
- IBKR session cookies
- Credentials or credential file contents
- Sensitive HTTP headers
- Local secret paths
- Raw prompt or broker text that has been classified as unsafe

Account identifiers may be stored raw only when explicitly configured for local
debugging; default behavior is to store a stable hash.
