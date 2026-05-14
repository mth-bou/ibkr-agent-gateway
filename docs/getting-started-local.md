# Getting Started: Local Read-Only Gateway

This guide tracks `specs/001-gateway-mvp-spec`.

The first implementation phase is local, single-user, and read-only. It targets
a manually authenticated Interactive Brokers Client Portal Gateway session and a
fake backend for offline validation.

## Initial Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## US1 Commands

The first story validates gateway health, broker session visibility, safe manual
session requirements, and safe account discovery. In the current offline
implementation these commands use the fake backend fixtures under
`tests/fixtures/cpapi/` when no config loader is provided.

```bash
ibkr-agent health --json
ibkr-agent backend status --json
ibkr-agent session requirements --json
ibkr-agent accounts list --json
```

Expected behavior:

- `health` returns gateway status and confirms read-only mode.
- `backend status` returns broker backend status without cookies, tokens,
  headers, or raw session material.
- `session requirements` returns `manual_action: none` when the fake session is
  usable, or a safe manual action when a missing/expired session fixture is
  used by tests.
- `accounts list` returns account id, HMAC audit correlation id, account mode,
  optional safe label, and base currency only.

Order preview, order submit, order cancel, remote MCP, sidecar relay, and live
trading remain out of scope for this phase.

## Read-Only Data Commands

```bash
ibkr-agent account summary --account DU1234567 --json
ibkr-agent portfolio snapshot --account DU1234567 --json
ibkr-agent positions list --account DU1234567 --json
ibkr-agent contracts search AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent contracts resolve AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent market snapshot --contract-id 265598 --json
ibkr-agent market bars --contract-id 265598 --duration "1 D" --bar-size "5 mins" --json
ibkr-agent orders list --account DU1234567 --json
ibkr-agent orders status --account DU1234567 --broker-order-id 123 --json
ibkr-agent executions list --account DU1234567 --json
```

## Local MCP and Audit Review

```bash
ibkr-agent mcp serve --transport stdio --json
ibkr-agent audit tail --limit 20 --json
```

MCP is local stdio only in this MVP. `ibkr_audit_tail` requires
`ibkr:audit:read`. Remote MCP, OAuth/OIDC, sidecar relay, direct broker OAuth,
and all trading writes remain later specs.
