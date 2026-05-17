# Paper Orders

Paper orders are the first paper-only write workflow. They do not enable live
trading.

## Required Gates

Paper submit and cancel require:

- explicit paper trading enablement
- a paper account in the allowlist
- paper submit or cancel scope
- a persisted approval record for submit
- an idempotency key
- audit events for approval, submit, cancel, and lifecycle transitions
- a configured paper writer for broker-side submit/cancel when validating
  against Client Portal Gateway

Live accounts and generic live submit/cancel tools remain absent or refused.

## CLI

Create a local approval record:

```bash
ibkr-agent approvals create \
  --account DU1234567 \
  --preview-id <preview_id> \
  --ttl-seconds 300 \
  --json
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
Approvals are bound to one persisted preview and are consumed after a
successful submit; reuse with a fresh idempotency key returns
`APPROVAL_CONSUMED`.
Without `--enable-paper`, paper submit and cancel return a typed disabled
refusal. Approval and idempotency records are persisted in the configured audit
SQLite database so replays remain stable across CLI invocations.

The CLI defaults to `LocalCandidatePaperWriter` for offline smoke tests. Runtime
deployments can wire `ClientPortalPaperWriter` to exercise the real paper
account path through the Client Portal Gateway before live trading is enabled.

## MCP

The production stdio MCP registry is read-only. Paper submit/cancel are
available through the explicit CLI and SDK workflow, not through default local
MCP discovery. Generic `ibkr_order_submit`, `ibkr_order_cancel`, and
`ibkr_order_approve` remain forbidden.

## Idempotency

Paper submit/cancel requests must include idempotency keys. Replaying the same
key with the same canonical request is treated as replay. Reusing the same key
with a different request is refused with `PAPER_IDEMPOTENCY_CONFLICT`.
Before the broker writer is called, the gateway stores a pending idempotency
record. If the process crashes before the final receipt is recorded, the same
key refuses retry until recovery resolves the pending broker-side state.
At CLI startup, pending submit records are recovered by checking broker order
status with the original idempotency key, which is sent to IBKR as `cOID`.
