# Audit Log

The gateway records security-relevant activity as redacted, append-only SQLite
rows. Audit is used for read operations, scope denials, preview/risk decisions,
paper lifecycle transitions, remote auth events, sidecar forwarding, and
live submit/cancel lifecycle events.

## Storage

The SQLite schema lives at
`src/internal/audit/migrations/0001_audit_events.sql`.

`SqliteAuditWriter` configures WAL journaling, writes redacted payload JSON, and
stores a chained HMAC hash for tamper-evidence across appended rows.

## Review and Export

```bash
ibkr-agent audit tail --limit 100 --json
ibkr-agent audit tail --database-url sqlite:/path/to/audit.db --limit 100 --json
ibkr-agent audit export --database-url sqlite:/path/to/audit.db --limit 500 --json
```

MCP clients use `ibkr_audit_tail` with `ibkr:audit:read`.

## Redaction

Audit payloads must not store:

- bearer tokens;
- cookies;
- credentials;
- sensitive headers;
- local secret paths;
- raw Client Portal Gateway session material;
- raw account ids.

Account and token correlation use HMAC-SHA256. Free-form audit metadata is
scrubbed by sensitive field name before persistence.

Denied, refused, failed, and completed operations keep a consistent correlation
shape so review can reconstruct what happened without exposing broker secrets.
