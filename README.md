# IBKR Agent Gateway

Rust-first, provider-neutral gateway for exposing Interactive Brokers data and
future order workflows through deterministic services, scoped MCP tools, CLI
operator commands, and redacted audit logs.

This is an unofficial open-source project and is not affiliated with,
endorsed by, or supported by Interactive Brokers.

## Current MVP

The implemented MVP is local, single-user, and read-only. It provides:

- typed Rust crates for domain, config, auth/scopes, audit, CPAPI mapping,
  backend abstraction, MCP, and CLI
- fake Client Portal Gateway fixtures for offline validation
- CLI commands for health, session, accounts, portfolio, positions, contracts,
  market data, read-only orders, executions, and audit tail
- local MCP stdio tool discovery for read-only broker tools
- local scope enforcement and HMAC/redaction audit helpers

## Roadmap

The project is governed by `specs/000-project-roadmap/`.

Implementation starts with `specs/001-gateway-mvp-spec/`, the local read-only
MVP. Later specs add order preview, paper trading, remote OAuth MCP, sidecar
relay, provider compatibility, live trading gates, and operations hardening in
that order.

## Current Non-Goals

- no order preview in the read-only MVP
- no order submit or cancel
- no remote public MCP endpoint
- no sidecar relay
- no provider-specific broker logic
- no live trading

## Quickstart

Use the local fake backend path first:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
ibkr-agent health --json
ibkr-agent accounts list --json
ibkr-agent mcp serve --transport stdio --json
ibkr-agent audit tail --limit 20 --json
```

## Target Package Usage

The packaging refactor is moving the project toward one user-facing package:

```bash
cargo install ibkr-agent-gateway
ibkr-agent health --json
```

and one SDK-style dependency:

```bash
cargo add ibkr-agent-gateway
```

The intended public Rust API is documented in `docs/public-api.md`. The package
is not publishable yet; `publish = false` remains in place until the internal
workspace crates are migrated behind the root package facade.

Detailed flows:

- `docs/getting-started-local.md`
- `docs/tools.md`
- `docs/mcp-local.md`
- `docs/audit-log.md`
- `docs/testing.md`
- `docs/public-api.md`
