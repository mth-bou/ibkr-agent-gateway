# Changelog

All notable changes to `ibkr-agent-gateway` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Remote MCP rate limiting is now operator-configurable through
  `remote_mcp.rate_limit_max_requests` and
  `remote_mcp.rate_limit_window_seconds` in CLI YAML and
  `RemoteMcpConfig`.

### Security

- Remote MCP HTTP serving now enforces a configurable
  `remote_mcp.max_connections` cap and returns `503` when the active
  connection limit is reached.
- Remote MCP limit fields validate fail-closed and keep serde defaults for
  older direct `RemoteMcpConfig` payloads.

## [0.2.0] - 2026-05-18

### Breaking Changes

- `ValidatedOrder` now carries the source `preview_id`; approvals must be
  created for an existing persisted preview with
  `ibkr-agent approvals create --preview-id <preview_id>`.
- Paper and live submit results now include the consumed approval record so
  callers can persist one-time approval consumption after successful submit.
- `submit_paper_order` and `cancel_paper_order` are now async and require a
  `PaperOrderWriter` implementation, matching the live writer boundary.

### Security

- Paper and live submit gates now verify `approval.preview_id` against the
  submitted order's source preview and refuse mismatches with
  `APPROVAL_PREVIEW_MISMATCH`.
- Successful paper and live submits mark approvals as consumed; later submit
  attempts with a fresh idempotency key are refused with `APPROVAL_CONSUMED`.
- CLI live submit/cancel now reuse the validated runtime
  `live_trading.allowed_accounts` and `live_trading.risk_policy_id` instead of
  deriving a live allowlist from the command target account.
- CLI audit commands now require `ibkr:audit:read` and append redacted audit
  events for tail/export/verify actions.
- CLI config loading now enforces the localhost-only TLS bypass rule before
  constructing Client Portal Gateway clients.
- Live cancel now preserves broker lifecycle state instead of forcing every
  accepted cancel response to `cancelled`; pending cancels remain in the live
  reconciliation backlog until a terminal status is observed.
- CLI config loading now validates live audit retention when live trading is
  enabled and rejects unknown YAML fields instead of silently ignoring them.

### Added

- `PaperOrderWriter` with local-candidate, refusing, and Client Portal Gateway
  implementations for end-to-end paper submit/cancel validation.
- Write-ahead idempotency records for order writer calls, including
  `pending_writer` and `failed_after_writer` states so a retry cannot silently
  re-call the broker after a crash window.
- Startup recovery scans pending order idempotency records and completes them
  from broker order status when the IBKR `cOID`/idempotency key can be found.
- Live submit now resolves hard-limit policies from a server-side
  `LivePolicyRegistry` instead of accepting caller-supplied policy objects.
- Live risk policies can enforce a market price collar and maximum quote age,
  refusing missing, stale, or out-of-band market snapshots before submit.
- Explicit live MCP handlers can submit or cancel live orders using only
  server-side approval, preview, policy, writer, and audit state.
- Live submits now enter a SQLite reconciliation backlog, and a one-shot live
  reconciler polls broker order status, records lifecycle transitions, and
  removes terminal orders from the backlog.
- `ibkr-agent audit verify` now scans the full audit HMAC chain and reports the
  first broken sequence for monitoring pipelines.
- The CLI live smoke commands now support
  `--live-broker {local-candidate|client-portal|refusing}`.
- Live submit rate counters for CLI and MCP are derived from durable audit
  workflow state before risk gates run, so caller-supplied counters cannot
  bypass frequency/session limits.
- CLI config loading now accepts complete `remote_mcp` configuration, including
  `token_id_hmac_secret_env`, so the HTTP MCP describe path uses the same
  validated runtime config as the gateway.

### Fixed

- Client Portal live cancel no longer treats active `Submitted` /
  `PreSubmitted` statuses as accepted cancellation states.
- Client Portal reply-chain confirmation now honors the configured
  `max_reply_rounds` exactly instead of allowing one extra confirmation.
- MCP live submit/cancel handlers now rely on durable SQLite order idempotency
  instead of constructing a per-call in-memory idempotency store.
- `LiveTradingGate::risk_policy_pass` now reflects the actual risk decision
  while preserving the specific `LIVE_LIMIT_REFUSED` error for risk refusals.
- Paper and live submit now mark approvals as consumed after the writer call
  succeeds.
- Backend order-status lookup naming now reflects that Client Portal recovery
  accepts either a broker order id or an idempotency/client order id.

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

[Unreleased]: https://github.com/mth-bou/ibkr-agent-gateway/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/mth-bou/ibkr-agent-gateway/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mth-bou/ibkr-agent-gateway/releases/tag/v0.1.0
