# Changelog

All notable changes to `ibkr-agent-gateway` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-05-19

### Added

- Added wiremock CPAPI contracts for contextual read endpoints covering options,
  greeks, market depth, scanners, news, fundamentals, market sessions/holidays,
  FX rates, transfer history, and query-value encoding.
- New atomic audit methods `complete_order_workflow` and
  `complete_live_order_workflow` regroup idempotency record completion, live
  reconciliation backlog updates, and approval consumption in a single SQLite
  transaction held on one connection through `Pool::begin_with("BEGIN IMMEDIATE")`.

### Changed

- `ScopeSet::local_with_preview` and `ScopeSet::local_with_paper` now strictly
  enforce the per-tier scope subset (read + preview, and read + preview + paper
  respectively) instead of accepting any local scope. `local_with_live` remains
  the broad-acceptance constructor and is now the explicit constructor used by
  `remote_auth_context_from_input` to preserve the historical remote OAuth scope
  surface.

### Fixed

- Security CI now runs secret regression tests with the internal test-support
  feature enabled, and all integration tests that depend on hidden
  `testing` exports are declared with matching required features.
- Bracket submits now reload a persisted bracket preview record and reject
  approvals mixed from different bracket groups instead of reconstructing an
  arbitrary group at submit time.
- `ibkr_paper_bracket_order_submit` now uses durable SQLite idempotency,
  persists replay payloads, and consumes all three approvals after a successful
  grouped paper submit.
- Invalid MCP modify requests with no bounded changes are rejected before
  writing pending idempotency state, so a reused idempotency key is not poisoned
  by pre-writer validation failures.
- Remote MCP live-tool discovery now treats `ibkr:orders:live:modify` as a live
  tool-enabling scope, and remote authorization preserves valid `x-request-id`
  and `mcp-session-id` correlation headers.
- Paper submit, cancel, and modify lifecycle records now reflect the broker
  receipt instead of hardcoded `Submitted`/`Cancelled`/`Open` values. Paper
  cancel and modify refuse with `BROKER_RESPONSE_INVALID` when the broker does
  not accept and the response is not terminal, and paper submit maps
  `Rejected`/`Refused`/`Inactive` broker statuses to `Refused`.
- Order workflow completion is now atomic: a crash window between a successful
  writer call and approval consumption could previously leave an approval in
  `Approved` status, allowing the same approval to be replayed with a different
  idempotency key. Live submit, live cancel, live modify, paper submit, and
  paper/live bracket submit all migrate to the transactional audit methods.
- Sequential live bracket writer now reports the broker order ids of every
  already-submitted leg in the error message, `user_action`, and `tracing` log
  when a later leg fails, so operators can clean up orphaned parent or
  take-profit legs before retrying.
- Pre-writer `OrderValidationFailed` errors no longer keep the pending
  idempotency record around as `failed_after_writer`; the recovery backlog only
  receives records that actually reached the broker writer.

### Documentation

- Documented that contextual read CPAPI paths are gateway adapter contracts,
  not proof of endpoint availability or naming in the deployed IBKR Client
  Portal Gateway build.
- Documented that live session-notional gates are deterministic limit-price
  exposure counters, so market, stop, and trailing-stop orders without a
  `limit_price` are bounded by the count, quantity, symbol, asset class,
  price-collar, and quote-freshness gates rather than session-notional
  arithmetic.
- `submit_live_group_order` now carries a `# Safety` doc comment that records
  the caller invariants (approved unexpired bracket, trusted live limit policy,
  per-leg contexts) that the MCP handler enforces today.
- `handle_http_mcp_request` is documented as the one-shot entry point that
  rebuilds runtime state per call; long-lived servers should construct
  `HttpMcpRuntime` once and call `handle_http_mcp_request_with_runtime`.

## [0.4.0] - 2026-05-19

### Added

- Expanded the scope-filtered MCP surface to 45 explicit tools, including PnL,
  order history, account metadata, safety visibility, advanced market reads,
  contextual reads, MCP approval creation, order modify, and bracket workflows.
- Added stop, stop-limit, trailing-stop, and market order representations with
  live policy refusing market execution by default.
- Added live bracket MCP handling with per-leg approval loading, live limit
  checks, durable pending idempotency, replay payload persistence, and approval
  consumption after successful submit.
- Added freshness/source metadata to options, greeks, market depth, and scanner
  response models.

### Changed

- `ibkr_live_order_modify` now requires an approved replacement preview through
  `approval_id` and `preview_id`, applies live limits to the approved order, and
  records broker-derived lifecycle status instead of hardcoding the result.
- MCP approval creation now clamps approval TTLs to the same bounded window as
  CLI approval creation and records the caller identity from the active MCP
  runtime context.
- Compatibility discovery with live enabled now includes
  `ibkr_live_bracket_order_submit` consistently with the local registry.

### Fixed

- Rejected ambiguous modify requests that specify stop and trailing auxiliary
  prices together instead of silently overwriting `auxPrice`.
- Bounded Client Portal order-history limits at the backend request model
  boundary, not only in the MCP parser.
- Removed the unimplemented OCA tools from the Spec 009 MCP contract and
  documented OCA as future scope requiring a dedicated writer and tests.

## [0.3.0] - 2026-05-19

### Added

- MCP stdio transport now routes `ibkr_order_preview`,
  `ibkr_paper_order_submit`, `ibkr_paper_order_cancel`,
  `ibkr_live_order_submit`, and `ibkr_live_order_cancel` through the same
  scope-filtered registry and audit path as the read tools, so an MCP client
  can drive the full preview/paper/live workflow without going through the
  CLI.
- Remote MCP HTTP transport now binds a real listener and serves JSON-RPC
  `POST /mcp` plus `GET /.well-known/oauth-protected-resource` with
  OAuth/OIDC bearer validation, JWKS caching, scope-filtered tool discovery,
  and the same handlers as stdio. The previous CLI path only printed a
  configuration description.
- Remote MCP HTTP listener accepts concurrent connections through a
  cooperative `tokio::select!` poll loop with a shared JWKS cache, instead of
  serving one connection at a time.
- New sidecar relay workflow commands: `ibkr-agent sidecar session create`
  and `ibkr-agent sidecar relay accept` validate relay session creation and
  sanitize forwarded payloads before they reach the local Client Portal
  Gateway.
- CLI YAML config now loads the sidecar relay configuration block
  (`sidecar.enabled`, `sidecar.remote_relay_url`,
  `sidecar.local_client_portal_base_url`,
  `sidecar.heartbeat_interval_seconds`,
  `sidecar.heartbeat_timeout_seconds`) and the matching
  `safety.sidecar_enabled` independent safety flag, and validates them at
  startup with the same fail-closed rules as the SDK
  `GatewayConfiguration`.
- Remote MCP rate limiting is now operator-configurable through
  `remote_mcp.rate_limit_max_requests` and
  `remote_mcp.rate_limit_window_seconds` in CLI YAML and
  `RemoteMcpConfig`.

### Changed

- `ValidatedOrder` now carries the resolved `symbol` and `asset_class` from
  the persisted preview so live policy `allowed_symbols`,
  `allowed_asset_classes`, and session-notional checks evaluate the actual
  previewed instrument instead of hardcoded values.
- `build_validated_order` now accepts a `&ContractCandidate` instead of just
  a `ContractId` so symbol and asset-class metadata stay attached to the
  validated order.
- Live submit session notional is now derived from completed live
  idempotency records in the durable audit store, joined per session
  currency, instead of being seeded to zero by the caller.
- Pending writer cleanup logic (`handle_pending_order_error` and
  `is_writer_boundary_error`) is now centralized in
  `internal::orders::pending` and shared by every CLI and MCP write path.

### Fixed

- MCP live submit and cancel no longer write a second redundant audit event
  on top of the registry-level audit, so each live tool call appends exactly
  one MCP tool event.

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

[Unreleased]: https://github.com/mth-bou/ibkr-agent-gateway/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/mth-bou/ibkr-agent-gateway/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/mth-bou/ibkr-agent-gateway/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/mth-bou/ibkr-agent-gateway/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/mth-bou/ibkr-agent-gateway/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mth-bou/ibkr-agent-gateway/releases/tag/v0.1.0
