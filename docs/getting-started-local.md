# Getting Started: Local Read-Only Gateway

This guide tracks `specs/001-gateway-mvp-spec`.

The first implementation phase is local, single-user, and read-only. It targets
a manually authenticated Interactive Brokers Client Portal Gateway session and a
fake backend for offline validation.

## Initial Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
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
