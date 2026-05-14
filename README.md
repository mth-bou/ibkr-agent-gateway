# IBKR Agent Gateway

Rust-first, provider-neutral gateway for exposing Interactive Brokers data and
future order workflows through deterministic services, scoped MCP tools, CLI
operator commands, and redacted audit logs.

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
