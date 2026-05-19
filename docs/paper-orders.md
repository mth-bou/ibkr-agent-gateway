# Paper Orders

Paper orders are the first paper-only write workflow. They do not enable live
trading.

## Required Gates

Paper submit, cancel, and modify require:

- explicit paper trading enablement
- a paper account in the allowlist
- paper submit, cancel, or modify scope
- a persisted approval record for submit
- an idempotency key
- audit events for approval, submit, cancel, modify, and lifecycle transitions
- a configured paper writer for broker-side submit/cancel/modify when validating
  against Client Portal Gateway

Paper workflows do not enable live trading. Live submit/cancel/modify use the
separate live-gated commands and MCP tools, with independent config, scope,
approval, risk, kill switch, audit, and paper-to-live gates.

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

The MCP registry advertises explicit paper tools when their scopes are present
in the active local scope set or remote bearer token grant:

| Tool | Scope |
|------|-------|
| `ibkr_paper_order_submit` | `ibkr:orders:paper:submit` |
| `ibkr_paper_order_cancel` | `ibkr:orders:paper:cancel` |
| `ibkr_paper_order_modify` | `ibkr:orders:paper:modify` |

Paper submit still requires a persisted approval for a preview, an idempotency
key, and paper trading enablement. Paper cancel requires an idempotency key and
paper cancel scope. Paper modify requires `account_id`, `broker_order_id`,
`idempotency_key`, and at least one bounded change (`quantity`, `limit_price`,
`stop_price`, `time_in_force`, `trailing_amount`, or `trailing_percent`).
Generic `ibkr_order_submit`, `ibkr_order_cancel`, `ibkr_order_modify`, and
`ibkr_order_approve` remain forbidden.

## Idempotency

Paper submit/cancel/modify requests must include idempotency keys. Replaying the same
key with the same canonical request is treated as replay. Reusing the same key
with a different request is refused with `PAPER_IDEMPOTENCY_CONFLICT`.
Before the broker writer is called, the gateway stores a pending idempotency
record. If the process crashes before the final receipt is recorded, the same
key refuses retry until recovery resolves the pending broker-side state.
At CLI startup, pending submit records are recovered by checking broker order
status with the original idempotency key, which is sent to IBKR as `cOID`.
