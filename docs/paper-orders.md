# Paper Orders

`specs/003-paper-submit-approval` adds the first paper-only write workflow. It
does not enable live trading.

## Required Gates

Paper submit and cancel require:

- explicit paper trading enablement
- a paper account in the allowlist
- paper submit or cancel scope
- a persisted approval record for submit
- an idempotency key
- audit events for approval, submit, cancel, and lifecycle transitions

Live accounts and generic live submit/cancel tools remain absent or refused.

## CLI

Create a local approval record:

```bash
ibkr-agent approvals create --account DU1234567 --ttl-seconds 300 --json
```

Submit a paper order candidate:

```bash
ibkr-agent orders submit \
  --account DU1234567 \
  --approval-id <approval_id> \
  --idempotency-key paper-submit-001 \
  --enable-paper \
  --json
```

Cancel a paper order candidate:

```bash
ibkr-agent orders cancel \
  --account DU1234567 \
  --broker-order-id paper-order-local \
  --idempotency-key paper-cancel-001 \
  --enable-paper \
  --json
```

Without `--approval-id`, paper submit returns `PAPER_APPROVAL_REQUIRED`.
Without `--enable-paper`, paper submit and cancel return a typed disabled
refusal. Approval and idempotency records are persisted in the configured audit
SQLite database so replays remain stable across CLI invocations.

## MCP

The production stdio MCP registry is read-only. Paper submit/cancel are
available through the explicit CLI and SDK workflow, not through default local
MCP discovery. Generic `ibkr_order_submit`, `ibkr_order_cancel`, and
`ibkr_order_approve` remain forbidden.

## Idempotency

Paper submit/cancel requests must include idempotency keys. Replaying the same
key with the same canonical request is treated as replay. Reusing the same key
with a different request is refused with `PAPER_IDEMPOTENCY_CONFLICT`.
