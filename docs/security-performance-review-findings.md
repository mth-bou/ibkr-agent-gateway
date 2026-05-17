# Security and Performance Review Findings

Date: 2026-05-15

Scope: application code in the Rust workspace after specs 001-008, with focus on security, performance, correctness, and Rust best practices.

Review basis:

- `rust-core-majiayu000`: used as the general Rust review checklist for explicit error handling, type-driven APIs, dependency audit, testing, and performance validation.
- `rust-pro-v3` / `rust-pro-v2`: used as the advanced Rust checklist for async/runtime choices, memory safety, performance hotspots, API design, error propagation, and production readiness.
- `rust-async-patterns-v2`: used as the async checklist for timeout handling, async error propagation, task/concurrency boundaries, and avoiding unbounded waits.
- Repo-native checks: `cargo fmt --check`, `cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings`, `cargo test --workspace --features unstable-internal-test-support`, and `cargo audit --deny warnings`.

The local Rust skill packages themselves are intentionally out of scope for this file.

## Findings

### 1. Remote OAuth validation is not production-safe for real OIDC

Severity: High

Status: Fixed. The validator now supports RS256 JWKS keys while keeping HS256 only for deterministic local tests. RS256 happy-path and mismatched `kid` tests were added.

Files:

- `crates/ibkr-oauth/src/validator.rs`
- `crates/ibkr-oauth/src/jwks.rs`

Evidence:

- `validate_bearer_jwt` only accepts `HS256`.
- `Jwk` models `oct` symmetric keys and `verify_signature` expects symmetric key material from JWKS.

Risk:

Public OIDC providers normally expose public keys through JWKS, not shared HMAC secrets. This blocks real provider compatibility or encourages unsafe shared-secret distribution for remote MCP.

Recommended fix:

- Add asymmetric JWK support for the algorithms required by target providers, likely `RS256` first and then `ES256`/EdDSA if needed.
- Keep strict checks for `kid`, `alg`, issuer, audience/resource, `exp`, `nbf`, and accepted scopes.
- Keep `HS256` only for deterministic tests if still useful, clearly marked as test/local-only.

Acceptance criteria:

- Remote OAuth tests cover valid and invalid asymmetric JWTs.
- Tokens signed with unknown `kid`, wrong `alg`, wrong issuer, wrong audience, expired claims, future `nbf`, or missing scope are rejected.
- No production config requires symmetric key material from public JWKS.

### 2. Token audit hashing uses a repo-visible static key

Severity: High

Status: Fixed. Remote MCP now requires `remote_mcp.token_id_hmac_secret` when enabled, and token-id audit hashes use the configured secret instead of a repo-visible static key.

File:

- `crates/ibkr-mcp/src/http_auth.rs`

Evidence:

- `TOKEN_ID_HASH_SECRET` is a hard-coded byte string.
- The constant is used as `token_id_hmac_secret` for remote token id audit hashes.

Risk:

Audit token-id hashes become predictable and comparable across deployments. That weakens the intended privacy boundary for audit correlation.

Recommended fix:

- Move the token-id HMAC secret to configuration or environment.
- Reuse the existing pattern documented for `IBKR_AUDIT_HMAC_SECRET`, or add a dedicated `IBKR_REMOTE_TOKEN_HMAC_SECRET`.
- Validate that the secret is present when remote MCP is enabled.

Acceptance criteria:

- Remote MCP config fails closed when enabled without a token hash secret.
- Tests prove the raw token and raw `jti` never appear in audit output.
- Different configured secrets produce different token-id hashes for the same `jti`.

### 3. Idempotency is modeled but not enforced in order flows

Severity: High

Status: Fixed. Paper submit/cancel and live submit/cancel now require an `IdempotencyStore`, hash canonical request inputs, replay identical requests, and reject conflicting reuse.

Files:

- `crates/ibkr-orders/src/idempotency.rs`
- `crates/ibkr-orders/src/paper_submit.rs`
- `crates/ibkr-orders/src/paper_cancel.rs`
- `crates/ibkr-orders/src/live_submit.rs`
- `crates/ibkr-orders/src/live_cancel.rs`

Evidence:

- `IdempotencyStore::record_or_replay` exists.
- Paper submit/cancel and live submit/cancel accept an `IdempotencyKey`, but the submit/cancel flows do not call the store.
- Live submit marks the idempotency gate as `true` once the key was parsed.

Risk:

Duplicate submit/cancel requests can produce multiple lifecycle records and, once broker adapters are wired, could duplicate broker-side operations. This is especially dangerous for live trading.

Recommended fix:

- Make idempotency enforcement part of the order service boundary, not a test-only helper.
- Hash canonical request inputs and call `record_or_replay` before any broker-facing operation.
- Persist idempotency records in the same durability tier as order lifecycle/audit for paper/live flows.

Acceptance criteria:

- Same idempotency key plus same canonical request returns the same recorded decision/result.
- Same idempotency key plus different canonical request fails closed.
- Live submit/cancel gates fail if idempotency storage is unavailable.
- Tests cover paper submit, paper cancel, live submit, and live cancel idempotency.

### 4. CPAPI endpoint URLs are built with raw string interpolation

Severity: Important

Status: Fixed. CPAPI endpoints are now built from structured path segments and query pairs, with validation for unsafe path/query input.

File:

- `crates/ibkr-cpapi/src/client.rs`

Evidence:

- Account ids, symbols, contract ids, duration, and bar size are interpolated directly into paths and query strings.
- Current string id constructors only reject empty values.

Risk:

Malformed user-controlled input can alter query semantics or produce invalid endpoint URLs. This is not SQL injection, but it is still an external-boundary encoding issue and can cause broker request confusion.

Recommended fix:

- Build paths and query strings with URL APIs instead of `format!`.
- Use `Url::path_segments_mut` for path components and `query_pairs_mut` for query parameters.
- Tighten identifier validation for account ids, contract ids, symbols, duration, and bar size.

Acceptance criteria:

- Inputs containing `?`, `&`, `/`, whitespace, or control characters are rejected or encoded deterministically.
- Tests cover account summary, contract search, market snapshot, historical bars, orders, and executions URL construction.

### 5. JWKS fetching lacks timeout, status handling, size bound, and cache

Severity: Important

Status: Fixed. JWKS fetching now uses a bounded `reqwest::Client`, status checks, a maximum body size, and an in-memory TTL cache for hot-path validation.

File:

- `crates/ibkr-oauth/src/jwks.rs`

Evidence:

- `fetch_jwks` uses `reqwest::get`.
- It does not set explicit timeout, call `error_for_status`, bound response size, or cache keys.

Risk:

Remote auth can block on slow JWKS endpoints, parse unexpected error pages, or repeatedly fetch keys on hot paths. That conflicts with the roadmap target of remote auth checks under 50 ms excluding refresh.

Recommended fix:

- Use an injected `reqwest::Client` configured with connect/request timeouts.
- Call `error_for_status`.
- Bound the response body size before parsing.
- Cache JWKS by issuer/URL with TTL and refresh behavior.

Acceptance criteria:

- Tests cover timeout/unavailable JWKS, non-2xx status, invalid JSON, oversized body, and cached validation.
- Hot-path token validation uses cached JWKS and does not perform network I/O.

### 6. Live CLI wording can imply a real broker submission

Severity: Important

Status: Fixed. Live CLI human output now says `live order candidate recorded` / `live cancel candidate recorded`, with a unit test preventing broker-execution wording in the current local candidate path.

Files:

- `crates/ibkr-cli/src/commands/orders_live.rs`
- `crates/ibkr-orders/src/live_submit.rs`

Evidence:

- The CLI builds a dummy static AAPL live order.
- `submit_live_order` returns a local lifecycle record with `BrokerOrderId::from_static("live-order-local")`.
- The CLI prints `live order submitted`.

Risk:

Operators may believe a real live order was submitted or cancelled when the current implementation records a gated local candidate. For trading systems, wording ambiguity is a safety issue.

Recommended fix:

- Rename the current CLI output to `live order candidate recorded` or `live order dry-run passed gates` until a real broker adapter is wired.
- Require real broker submit/cancel adapters to return broker-generated order ids before using “submitted” or “cancelled”.

Acceptance criteria:

- CLI and MCP outputs distinguish local candidate/gate evaluation from broker-accepted live orders.
- Tests assert the safer wording for the current non-broker live path.

### 7. Performance coverage is too narrow for the implemented feature surface

Severity: Important

Status: Fixed. Performance budgets now cover cached remote OAuth validation, larger audit tail reads, live gate/risk/idempotency, and sidecar forwarded request safety.

File:

- `tests/integration_performance_budgets.rs`

Evidence:

- Current budget tests cover fake backend account listing and in-memory SQLite append/tail only.
- There is no budget coverage for remote auth cached validation, audit export over larger tails, CPAPI request construction, order risk/idempotency, or sidecar forwarding.

Risk:

Performance regressions can land in the more expensive paths while the existing perf test suite stays green.

Recommended fix:

- Add deterministic budget tests for:
  - cached remote OAuth validation,
  - audit export over a realistic tail size,
  - order preview/risk/live limit evaluation,
  - idempotency record/replay,
  - sidecar forwarded request hashing/safety validation.
- Keep broker network latency excluded from local budgets, as the roadmap specifies.

Acceptance criteria:

- Performance tests map directly to roadmap targets:
  - local read-only overhead under 500 ms p95 excluding broker latency,
  - audit write p95 under 50 ms locally,
  - order preview/risk under 250 ms excluding broker preview calls,
  - remote auth checks under 50 ms excluding JWKS refresh.

### 8. Workspace lints were declared but not applied to the published crate

Severity: Important

Status: Fixed. The root package now opts into the workspace lint profile, the profile keeps hard denies for unsafe code, `unwrap`, `expect`, `panic`, `todo`, and `dbg`, and the full test suite passes under `--all-targets`.

Files:

- `Cargo.toml`
- `.github/workflows/ci.yml`

### 9. Internal test harness was exposed in the default public API

Severity: Important

Status: Fixed. The internal `testing` module is now hidden behind the explicit `unstable-internal-test-support` feature. Normal consumers get the public SDK, CLI, MCP, config, audit, and orders modules without internal implementation re-exports.

Files:

- `Cargo.toml`
- `src/lib.rs`

### 10. Gateway recreated broker backends per operation

Severity: Important

Status: Fixed. `Gateway::new` now creates the selected backend once and stores it behind a shared trait object, so cloned gateway clients reuse the same backend instance instead of rebuilding it for each method call.

File:

- `src/public/gateway.rs`

### 11. Client Portal HTTP client lacked explicit timeouts

Severity: Important

Status: Fixed. `ClientPortalClient` now constructs a bounded `reqwest::Client` with explicit request and connect timeouts and propagates initialization failures as `GatewayError`.

Files:

- `src/internal/cpapi/client.rs`
- `src/internal/backend/factory.rs`

### 12. Fake backend performed synchronous filesystem reads from async methods

Severity: Important

Status: Fixed. `FakeFixtureStore::load_json` now uses `tokio::fs::read_to_string`, and all fake backend fixture reads are awaited from async paths.

Files:

- `Cargo.toml`
- `src/internal/backend/fake.rs`
- `tests/integration_contracts_market.rs`

## Checks Completed

- `cargo fmt --check`: passed.
- `cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace --features unstable-internal-test-support`: passed.
- `cargo test --workspace`: passed.
- `cargo audit --deny warnings`: passed.

## Notes for Fix Planning

No open findings remain in this review file. Re-run the release gates above before publishing a crates.io release candidate.
