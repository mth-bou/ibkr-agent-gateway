# Audit Retention and Backup

Audit data is operational evidence. It must remain redacted, immutable for live
write workflows, and exportable without raw broker secrets.

## Retention

- Read-only, preview, remote-auth, sidecar, provider, and paper events should be
  retained according to operator policy and storage capacity.
- Live write events must be retained for at least 2555 days.
- Live write events must be immutable after append.
- Purge jobs must require a prior export.
- Exports must include event ids, request/session correlation, tool names,
  scopes, decisions, result status, stable error codes, input/output hashes, and
  redaction metadata.

## Backup

For SQLite deployments:

1. Stop writers or use a consistent SQLite backup mechanism.
2. Export recent or full audit records with `ibkr-agent audit export --json`.
3. Store the JSONL payload and file hash together.
4. Verify the export contains no raw tokens, cookies, credentials, local paths,
   broker session material, or raw account ids.
5. Store backups in access-controlled storage separate from application logs.

## Restore and Replay

Replay must use redacted fixtures only. Restored audit exports are evidence for
debugging and regression tests; they are not a broker data cache and must not
recreate live broker sessions.
