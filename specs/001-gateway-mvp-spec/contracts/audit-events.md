# Contract: Audit Events

Audit events are append-only records for significant gateway behavior. They must be written for both allowed and denied/refused operations.

## Required Event Types

- `tool.called`
- `tool.denied_scope`
- `tool.completed`
- `tool.failed`
- `tool.refused`
- `backend.session.checked`
- `backend.session.changed`

## Event Shape

```json
{
  "event_id": "018f-example",
  "event_type": "tool.completed",
  "timestamp": "2026-05-14T10:15:30Z",
  "user_id": "local-user",
  "auth_source": "local_config",
  "session_id": "session-example",
  "request_id": "req-example",
  "account_id_hash": "hmac-sha256:example",
  "tool_name": "ibkr_accounts_list",
  "scopes": ["ibkr:accounts:read"],
  "decision": "allow",
  "result_status": "completed",
  "error_code": null,
  "input_hash": "hmac-sha256:example",
  "output_hash": "hmac-sha256:example",
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
- Raw Client Portal Gateway session material
- Raw prompt or broker text that has been classified as unsafe

## Hashing Rules

- Default account identifier representation is `hmac-sha256:<digest>`.
- HMAC key source is local generated, environment, or file-backed according to config.
- Raw SHA-256 is not the default for account identifiers or other low-entropy sensitive values.
- Raw account identifiers may be displayed in direct CLI command output only when the user explicitly requested account listing; audit storage still uses HMAC by default.

## Session Events

`backend.session.checked` records every explicit status/keepalive check.

`backend.session.changed` records transitions only when a previous externally visible state is known.

Example:

```json
{
  "event_id": "018f-session",
  "event_type": "backend.session.changed",
  "timestamp": "2026-05-14T10:15:30Z",
  "user_id": "local-user",
  "auth_source": "local_config",
  "session_id": "session-example",
  "request_id": "req-session",
  "account_id_hash": null,
  "tool_name": "ibkr_backend_status",
  "scopes": ["ibkr:health:read"],
  "decision": "allow",
  "result_status": "completed",
  "error_code": null,
  "input_hash": "hmac-sha256:example",
  "output_hash": "hmac-sha256:example",
  "redactions": [],
  "metadata": {
    "backend": "client_portal_gateway",
    "from": "usable",
    "to": "manual_action_required"
  }
}
```
