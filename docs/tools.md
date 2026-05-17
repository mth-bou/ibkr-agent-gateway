# CLI and Tool Commands

This document lists the developer-facing CLI flows. The CLI defaults to fake
fixtures when no config loader is wired.

## Health, Session, and Accounts

```bash
ibkr-agent health --json
ibkr-agent backend status --json
ibkr-agent session requirements --json
ibkr-agent accounts list --json
```

Session and backend outputs must not include cookies, bearer tokens, raw
headers, local secret paths, or raw broker session material.

## Portfolio, Positions, Contracts, and Market Data

```bash
ibkr-agent account summary --account DU1234567 --json
ibkr-agent portfolio snapshot --account DU1234567 --json
ibkr-agent positions list --account DU1234567 --json
ibkr-agent contracts search AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent contracts resolve AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent market snapshot --contract-id 265598 --json
ibkr-agent market bars --contract-id 265598 --duration "1 D" --bar-size "5 mins" --json
```

Account-scoped commands require an explicit account id. Contract resolution
fails closed when matches are ambiguous. Market data carries status so delayed,
stale, or unavailable data can be labeled or refused by policy.

## Read-Only Orders and Executions

```bash
ibkr-agent orders list --account DU1234567 --json
ibkr-agent orders status --account DU1234567 --broker-order-id 123 --json
ibkr-agent executions list --account DU1234567 --json
```

These commands inspect existing broker records only.

## Order Preview

Preview is non-executable and disabled unless explicitly enabled:

```bash
ibkr-agent orders preview \
  --account DU1234567 \
  --symbol AAPL \
  --side buy \
  --quantity 1 \
  --limit-price 100 \
  --currency USD \
  --enable-preview \
  --json
```

Without `--enable-preview`, the command returns `ORDER_PREVIEW_DISABLED`.

## Paper Orders

```bash
ibkr-agent approvals create --account DU1234567 --ttl-seconds 300 --json
ibkr-agent orders submit --account DU1234567 --idempotency-key paper-submit-001 --enable-paper --json
ibkr-agent orders cancel --account DU1234567 --broker-order-id paper-order-local --idempotency-key paper-cancel-001 --enable-paper --json
```

Paper submit/cancel require explicit paper enablement and an idempotency key.
Reusing the same key with different canonical request inputs is refused.

## Live-Gated Candidates

```bash
ibkr-agent orders live-submit \
  --account DU1234567 \
  --idempotency-key live-submit-001 \
  --enable-live \
  --live-scope \
  --open-kill-switch \
  --acknowledge-paper-to-live \
  --json
```

```bash
ibkr-agent orders live-cancel \
  --account DU1234567 \
  --broker-order-id live-order-local \
  --idempotency-key live-cancel-001 \
  --enable-live \
  --live-scope \
  --open-kill-switch \
  --acknowledge-paper-to-live \
  --json
```

Current live CLI paths record local lifecycle candidates after all gates pass.
They must not be described as broker-executed unless a real broker adapter
returns broker-generated order ids.

## Audit and MCP

```bash
ibkr-agent audit tail --limit 20 --json
ibkr-agent audit export --limit 500 --json
ibkr-agent mcp serve --transport stdio --json
```

Remote HTTP MCP and sidecar flows are disabled by default. See
[remote-mcp-oauth.md](remote-mcp-oauth.md) and [sidecar-relay.md](sidecar-relay.md).
