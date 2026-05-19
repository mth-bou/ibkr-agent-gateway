# CLI and Tool Commands

This document lists the developer-facing CLI flows. Without `--config`, the CLI
uses fake fixtures and a local SQLite audit/state file under the system temp
directory. With `--config`, a missing or invalid file fails closed.

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

## MCP-Only Maturity Reads

The MCP registry also exposes consultative and safety read tools when their
scopes are present:

| Tool | Scope |
|------|-------|
| `ibkr_pnl_daily` | `ibkr:portfolio:read` |
| `ibkr_pnl_realtime` | `ibkr:portfolio:read` |
| `ibkr_orders_history` | `ibkr:orders:read` |
| `ibkr_account_metadata` | `ibkr:accounts:read` |
| `ibkr_kill_switch_status` | `ibkr:health:read` |
| `ibkr_limits_status` | `ibkr:risk:read` |
| `ibkr_audit_export` | `ibkr:audit:export` |
| `ibkr_session_renew` | `ibkr:health:read` |

These tools are read-only or operational visibility tools. They do not submit,
cancel, modify, or approve broker orders.

Advanced market research tools are also MCP-only in this maturity increment:

| Tool | Scope |
|------|-------|
| `ibkr_options_chain` | `ibkr:options:read` |
| `ibkr_option_greeks` | `ibkr:options:read` |
| `ibkr_market_depth` | `ibkr:marketdata:depth:read` |
| `ibkr_scanner_run` | `ibkr:scanner:read` |

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
MCP preview additionally accepts stop, stop-limit, and trailing-stop candidates
with the required price or trailing fields for each order type. Market
candidates remain refused by policy.

## Paper Orders

```bash
ibkr-agent approvals create --account DU1234567 --preview-id <preview_id> --ttl-seconds 300 --json
ibkr-agent orders submit --account DU1234567 --approval-id <approval_id> --idempotency-key paper-submit-001 --enable-paper --json
ibkr-agent orders cancel --account DU1234567 --broker-order-id paper-order-local --idempotency-key paper-cancel-001 --enable-paper --json
```

Paper submit requires an approval id returned by `approvals create` for the
specific preview id. Paper submit/cancel/modify require explicit paper
enablement and an idempotency key. Reusing the same key with different canonical request
inputs is refused.

## Live-Gated Candidates

```bash
ibkr-agent orders live-submit \
  --account DU1234567 \
  --approval-id <approval_id> \
  --idempotency-key live-submit-001 \
  --enable-live \
  --live-scope \
  --open-kill-switch \
  --acknowledge-paper-to-live \
  --live-broker local-candidate \
  --json
```

```bash
ibkr-agent orders live-cancel \
  --account DU1234567 \
  --broker-order-id local-candidate-live-submit-001 \
  --idempotency-key live-cancel-001 \
  --enable-live \
  --live-scope \
  --open-kill-switch \
  --acknowledge-paper-to-live \
  --live-broker local-candidate \
  --json
```

Live submit, cancel, and modify run the full gate stack and then call a
`LiveOrderWriter`. `--live-broker` selects `local-candidate`,
`client-portal`, or `refusing`; the default returns deterministic
`local-candidate-*` ids. `client-portal` requires a config using
`broker.backend: client_portal_gateway`.

When `--config` is supplied, live commands use the validated
`live_trading.allowed_accounts` and `live_trading.risk_policy_id` from that
runtime config. The CLI invocation flags (`--enable-live`, `--live-scope`,
`--open-kill-switch`, and `--acknowledge-paper-to-live`) do not add the target
account to the allowlist and cannot replace the configured live policy.

Live submit rate counters and session notional are derived from durable audit
workflow state before the gate stack runs. Caller-supplied
`submitted_in_window`, `submitted_in_session`, and instrument context are not
trusted by CLI or MCP live paths.

## Audit and MCP

```bash
ibkr-agent audit tail --limit 20 --json
ibkr-agent audit export --limit 500 --json
ibkr-agent audit verify --json
ibkr-agent mcp serve --transport stdio --describe --json
ibkr-agent mcp serve --transport stdio
ibkr-agent mcp serve --transport http --describe --enable-remote-mcp --json
ibkr-agent --config config/remote.example.yaml mcp serve --transport http --enable-remote-mcp --bind 127.0.0.1:8080
```

`--describe` is a smoke check that prints the selected transport description and
exits. Without `--describe`, the stdio command runs the local MCP JSON-RPC loop.
It advertises only tools whose scopes are enabled by the current CLI config and
audits every `tools/call`.

Without `--describe`, the HTTP command binds the selected address, validates
OAuth/OIDC bearer tokens, serves `/.well-known/oauth-protected-resource`, and
routes JSON-RPC requests on `POST /mcp` through the same scoped tool handlers.

CLI audit commands require `ibkr:audit:read` in the current local scope set and
write their own redacted audit event after tail/export/verify completes.

Remote HTTP MCP and sidecar flows are disabled by default and fail closed
without explicit config or flags. See [remote-mcp-oauth.md](remote-mcp-oauth.md)
and [sidecar-relay.md](sidecar-relay.md).

## Sidecar Relay

```bash
ibkr-agent sidecar identity create --public-key <public-key> --display-name laptop --json
ibkr-agent sidecar pairing create --remote-instance-id remote-1 --sidecar-id sidecar-example --user-id local-user --ttl-seconds 300 --json
ibkr-agent sidecar session create --remote-instance-id remote-1 --sidecar-id sidecar-example --ttl-seconds 300 --json
ibkr-agent sidecar relay accept --remote-instance-id remote-1 --sidecar-id sidecar-example --tool-name ibkr_accounts_list --scope ibkr:accounts:read --payload-json '{}' --json
```

The sidecar commands expose the relay primitives: identity, pairing, session
creation, and sensitive-payload rejection for forwarded requests. They are not a
long-running daemon and do not automate Client Portal Gateway login.
