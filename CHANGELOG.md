# Changelog

All notable changes to `ibkr-agent-gateway` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-05-17

Initial public release.

### Added

- Single-crate Rust workspace exposing two entrypoints:
  - `ibkr-agent` — operator/developer CLI.
  - `ibkr_agent_gateway` — embeddable Rust library facade.
- Read-only broker integrations against the Interactive Brokers Client Portal
  Gateway: session status, accounts, portfolio, positions, contracts,
  market data snapshots, read-only orders, executions.
- Offline fake backend with fixtures under `tests/fixtures/cpapi` for fast
  development and deterministic CI.
- Local MCP stdio server with read-only tool discovery, scope-filtered tool
  catalog, and per-call audit.
- Remote MCP HTTP authorization primitives: OAuth/OIDC validation with
  RS256 JWKS, protected-resource metadata, generic auth denials, rate
  limiting.
- Order preview with deterministic risk checks and canonical fingerprints.
- Paper submit/cancel lifecycle with persisted approvals and idempotency
  keys.
- Live submit/cancel gates with allowlists, scopes, approval, risk policies,
  kill switch, audit-availability, and paper-to-live migration checklist.
- Live order writer trait with a Client Portal Gateway implementation, so
  live submit and cancel return broker-generated order ids when wired
  against a real Client Portal Gateway.
- Redacted audit storage (SQLite) with tail, export, and replay paths;
  HMAC-based account/audit correlation.
- Provider compatibility helpers for OpenAI, Anthropic, and generic MCP
  clients (read-only).
- Sidecar relay primitives: identity, pairing, heartbeat, and session
  binding.

### Security

- Workspace lints set `unsafe_code = forbid` and treat
  `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`,
  `clippy::todo`, and `clippy::dbg_macro` as `deny`.
- Broker credentials, cookies, bearer tokens, raw headers, and local session
  material are never returned to agents, CLI output, logs, fixtures, or
  audit payloads.
- Bundled `cargo audit` run against the released `Cargo.lock`: no known
  advisories across all transitive dependencies.

### Documentation

- Developer guide, production readiness checklist, public API reference,
  MCP local and remote OAuth guides, audit log and retention, order preview,
  paper and live runbooks, sidecar relay, and testing guide all shipped in
  `docs/`.

### Known Limitations

- The bundled live writer talks to the Client Portal Gateway's order
  endpoints. Consumers deploying live trading must validate the writer
  against their own paper environment before enabling live trading in
  production. The kill switch, allowlists, and migration checklist remain
  the operator's primary line of defense.
- The `unstable-internal-test-support` Cargo feature exposes internal
  test helpers. It is explicitly unstable and not part of the SDK's
  public API surface.

[Unreleased]: https://github.com/mth-bou/ibkr-agent-gateway/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/mth-bou/ibkr-agent-gateway/releases/tag/v0.1.0
