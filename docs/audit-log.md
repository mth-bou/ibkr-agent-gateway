# Audit Log

The read-only MVP stores audit records as append-only SQLite rows. Each row
contains a sequence id, event id, event type, timestamp, and a redacted JSON
payload.

## Storage

The SQLite schema lives in `crates/ibkr-audit/migrations/0001_audit_events.sql`.
Application code writes through `SqliteAuditWriter` and reads recent events with
the bounded `tail` query.

## Review

```bash
ibkr-agent audit tail --limit 100 --json
```

The local CLI accepts an optional SQLite database URL:

```bash
ibkr-agent audit tail --database-url sqlite:/path/to/audit.db --limit 100 --json
```

MCP clients use `ibkr_audit_tail` with `ibkr:audit:read`.

## Redaction

Audit payloads must not store tokens, cookies, credentials, sensitive headers,
local secret paths, or raw Client Portal Gateway session material. Account
correlation uses HMAC-SHA256 by default through the audit redaction helpers.

Denied scope, refused write-like tools, failed calls, and completed read-only
calls all keep the same correlation shape so later review can reconstruct what
was attempted without exposing broker secrets.
