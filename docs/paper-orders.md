# Paper Orders

`specs/003-paper-submit-approval` adds the first paper-only write workflow. It
does not enable live trading.

## Required Gates

Paper submit and cancel require:

- explicit paper trading enablement
- a paper account in the allowlist
- paper submit or cancel scope
- a prior approved preview for submit
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

Without `--enable-paper`, paper submit and cancel return a typed disabled
refusal.

## MCP

Paper tools use paper-specific names and scopes:

| Tool | Scope |
|------|-------|
| `ibkr_paper_order_submit` | `ibkr:orders:paper:submit` |
| `ibkr_paper_order_cancel` | `ibkr:orders:paper:cancel` |

Generic `ibkr_order_submit`, `ibkr_order_cancel`, and `ibkr_order_approve`
remain forbidden.

## Idempotency

Paper submit/cancel requests must include idempotency keys. Replaying the same
key with the same canonical request is treated as replay. Reusing the same key
with a different request is refused with `PAPER_IDEMPOTENCY_CONFLICT`.
