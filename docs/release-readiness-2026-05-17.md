# Release Readiness Review — 2026-05-17

Comprehensive readiness review of `ibkr-agent-gateway` v0.1.0 for production
deployment and publication on crates.io.

## Executive Summary

**Verdict: SHIP READY for crates.io v0.1.0 publication.**

**Verdict for unattended live trading: ARCHITECTURE READY** — live submit and
cancel now delegate the broker call to a `LiveOrderWriter`. The bundled
`ClientPortalLiveWriter` implementation talks to the Interactive Brokers
Client Portal Gateway, handles the reply-chain confirmation protocol, and
returns broker-generated order ids. Operational deployments must still
validate the writer against their own paper environment before flipping live
flags; the kill switch, allowlists, and migration checklist remain the
operator's primary line of defense.

All automated quality gates pass cleanly. No critical or high-severity issues
were found. LICENSE and CHANGELOG have been added at the repo root.

## Validation Matrix

| Gate | Command | Result | Notes |
|------|---------|--------|-------|
| Formatting | `cargo fmt --check` | ✅ PASS | Clean |
| Lints | `cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings` | ✅ PASS | 0 warnings, deny posture preserved |
| Tests | `cargo test --workspace --features unstable-internal-test-support` | ✅ PASS | 150 passed / 0 failed / 0 ignored across 59 binaries |
| Docs | `cargo doc --workspace --no-deps` | ✅ PASS | Builds cleanly |
| Package | `cargo package --allow-dirty --no-verify --list` | ✅ PASS | 253 files, 654.8 KiB raw / 143.9 KiB compressed |
| Publish dry-run | `cargo publish --dry-run --locked` | ✅ PASS | Only the expected "aborting upload due to dry run" |
| Security audit | `cargo audit` | ✅ PASS | 0 advisories across 293 transitive crates |

## Package Hygiene

### `Cargo.toml` metadata — complete

- `name`, `version`, `edition = "2024"`, `rust-version = "1.94.1"`
- `description`, `license = "MIT"`, `repository`, `readme = "README.md"`
- `keywords` (5 — at the crates.io maximum), `categories`
- `include` list explicitly enumerates only:
  - `Cargo.lock`, `Cargo.toml`, `README.md`
  - `config/local.example.yaml`, `docs/**`, `examples/**`, `rust-toolchain.toml`
  - `src/**`, `tests/**`

### Files excluded from the package — confirmed

The published tarball does NOT contain:

- `.agents/`, `.claude/`, `.codex/`, `.idea/`, `.vscode/`, `.specify/`
- `target/`, `.env*`
- `ibkr-agent-gateway-plan.md` (large internal planning artifact)
- `AGENTS.md`, `specs/` (verified via `cargo package --list`)

This is enforced by the explicit `include` list — a safer approach than
`exclude` because it fails closed.

### Binaries and library

- Library crate: `ibkr_agent_gateway` (publishes the SDK facade)
- Binary: `ibkr-agent` at `src/bin/ibkr-agent.rs`
- Both shipped together — `cargo install ibkr-agent-gateway` installs the CLI

## Public API Surface (`src/lib.rs`)

Clean, opinionated, and stable-ready:

- Top-level re-exports limited to `Gateway` and `GatewayConfig` (deliberate
  surface minimization).
- Domain modules behind `audit`, `config`, `mcp`, `orders` namespaces.
- `prelude` covers common imports.
- `internal` module is private — no internal types leak.
- `testing` module is gated behind `unstable-internal-test-support`
  and marked `#[doc(hidden)]` — it cannot be relied on by external consumers
  without opting into the explicitly-unstable feature.
- `cli` module is `#[doc(hidden)]` but `pub` — necessary because the binary
  links against it; not part of the SDK contract.
- All public struct fields and constructors carry `///` documentation.

## Safety Posture

The crate is intentionally aggressive about correctness:

- `unsafe_code = "forbid"` (workspace lint)
- `clippy::unwrap_used = "deny"`
- `clippy::expect_used = "deny"`
- `clippy::panic = "deny"`
- `clippy::todo = "deny"`
- `clippy::dbg_macro = "deny"`

The single `panic!` site in `src/` lives at `src/internal/backend/fake.rs:316`,
inside a `#[cfg(test)] mod tests` block that deliberately poisons a
`RwLock` to prove cache survival behavior. Not reachable in release code paths.

No residual `TODO` / `FIXME` / `HACK` / `XXX` markers in `src/` or `tests/`.

## Secrets and Redaction

No hardcoded secrets in source. The four `"password="` matches under `src/`
are all redaction patterns in:

- `src/internal/observability/metrics.rs`
- `src/internal/observability/logs.rs`
- `src/internal/audit/export.rs`
- `src/internal/audit/replay.rs`

These are blocklists that *prevent* `password=...` substrings from reaching
logs, metrics, audit exports, or replay tools — i.e. defensive code, not a
leak.

Two operator secrets are documented as required for production:

- `IBKR_AUDIT_HMAC_SECRET` — stable HMAC for account/audit correlation
- `IBKR_REMOTE_TOKEN_HMAC_SECRET` — HMAC for remote token-id audit hashes

Both are correctly sourced from environment, not hardcoded.

## Dependency Posture

- 293 transitive crates, 0 known vulnerabilities (RustSec advisory DB,
  1090 advisories loaded).
- TLS via `rustls` (`reqwest` uses `default-features = false` + `rustls-tls`)
  — no OpenSSL toolchain dependency on the host.
- `sqlx-sqlite` is pinned exactly to `=0.8.6` with the `bundled` feature —
  reproducible, no system SQLite dependency.
- `ring`, `hmac`, `sha2` for crypto primitives — well-audited.
- `secrecy` and `zeroize` are available in the workspace dependency set for
  in-memory secret handling.

## Release Profile

```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = "debuginfo"
```

Good defaults for a CLI/library binary: thin LTO trades a little build time
for a smaller, faster artifact; `codegen-units = 1` maximizes optimization;
`strip = "debuginfo"` keeps the binary small without removing symbols needed
for backtraces.

## Documentation

The `docs/` tree is complete and consistent with the source. Key documents
shipped in the package:

- `README.md` — quick start, CLI examples, developer checks, links
- `docs/developer-guide.md` — primary developer onboarding
- `docs/production-readiness.md` — operator deployment checklist
- `docs/public-api.md` — public SDK contract
- `docs/mcp-local.md`, `docs/remote-mcp-oauth.md`, `docs/sidecar-relay.md`
- `docs/scopes.md`, `docs/audit-log.md`, `docs/audit-retention.md`
- `docs/order-preview.md`, `docs/paper-orders.md`, `docs/paper-to-live.md`
- `docs/live-runbook.md`, `docs/incident-review.md`
- `docs/provider-compatibility.md`, `docs/provider-approval-ux.md`
- `docs/testing.md`, `docs/tools.md`

## Test Surface

150 tests across 59 binaries:

- **Contract tests** (15 binaries) — public API stability, MCP schemas, CLI
  surface, OAuth denials, paper/live separation
- **Integration tests** (31 binaries) — end-to-end workflows: accounts,
  audit (sqlite + scope denials + US1 events), backend status, keepalive,
  live gate refusals + idempotency + kill switch, MCP keepalive + redaction
  + scope denials + write refusals, order preview/lifecycle/audit, paper
  approval, performance budgets, portfolio positions, project boundaries,
  quickstart read-only, remote OAuth, sidecar audit/heartbeat/pairing,
  write refusals
- **Replay/snapshot tests** (8 binaries) — audit redaction replay, paper
  idempotency, remote token redaction, secret scan, sidecar secret scan,
  provider auth denial / redaction / schema snapshots
- **Log redaction** and **schema drift** binaries
- Unit tests inside `src/lib.rs` modules
- Doc-tests on the library facade

## Recommendations (Non-Blocking)

Two minor items to consider before the actual `cargo publish`:

### 1. Add a root `LICENSE` file (recommended)

`Cargo.toml` declares `license = "MIT"` and that is sufficient for crates.io
— it does not warn during the dry run. However:

- GitHub will not detect or display the MIT license in the repo UI without
  a file named `LICENSE` or `LICENSE-MIT`.
- REUSE compliance, SBOM tools, and some downstream packaging pipelines
  look for the file directly.
- It costs nothing to add and avoids future complaints.

Recommended action: drop the canonical MIT text (with `Copyright (c) 2026
Mathieu`) into `LICENSE` at the repo root. Optionally add it to the package
`include` list so it is also embedded in the crates.io tarball.

### 2. Add `CHANGELOG.md` (recommended)

For a v0.1.0 first release, a short CHANGELOG framing the initial scope
and the explicitly-unstable boundaries (live trading lifecycle candidates,
`unstable-internal-test-support` feature) helps downstream operators
calibrate their integration depth.

A "Keep a Changelog" formatted file with a single `## [0.1.0] - 2026-05-17`
section is enough.

### 3. (Optional) Move `ibkr-agent-gateway-plan.md` out of repo root

The 38 KB planning artifact at the root is *not* shipped in the package (the
`include` list excludes it), so it is harmless to crates.io publication.
However, it is visible in the GitHub repo. If it contains anything you would
rather keep internal, move it under `docs/internal/` and add a
`docs/internal/` entry to `.gitignore`, or rename and trim for a public
audience.

### 4. (Optional) Re-export key types at crate root for convenience

Today consumers must reach into `ibkr_agent_gateway::audit::*`,
`ibkr_agent_gateway::config::*`, etc. The `prelude` covers the common case.
This is a deliberate stability decision and is fine; just noting it for
future API discussions.

## Final Checklist Before `cargo publish`

When you are ready to push to crates.io:

```bash
# 1. Reconfirm the gates
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
cargo doc --workspace --no-deps
cargo audit

# 2. Reconfirm package contents
cargo package --allow-dirty --no-verify --list
cargo publish --dry-run --locked

# 3. Tag the release
git tag -a v0.1.0 -m "v0.1.0 — initial crates.io release"
git push origin v0.1.0

# 4. Publish
cargo publish --locked
```

## Production Readiness (Operational Deployment)

Re-read [production-readiness.md](production-readiness.md) before any
operational deployment.

Live submit and cancel now go through a pluggable
[`LiveOrderWriter`](../src/internal/orders/live_writer.rs):

- production deployments wire
  [`ClientPortalLiveWriter`](../src/internal/cpapi/live_writer.rs) against a
  configured `ClientPortalClient`; the writer posts orders, handles the
  IBKR reply chain, and returns broker-generated order ids;
- the CLI ships wired to `LocalCandidateLiveWriter` because it is a smoke
  harness for the gate stack, not a production live-trading entrypoint;
- `RefusingLiveWriter` is available as a fail-closed default for
  environments that have not been validated.

The kill switch, allowlists, scopes, approvals, idempotency keys, audit
availability, and paper-to-live checklist remain mandatory — they all run
before the writer is invoked. The contract test suite
(`tests/contract_cpapi_live_writer.rs`, 11 tests against a mocked Client
Portal Gateway) exercises the happy path, the reply confirmation chain, the
reply-depth limit, market/limit refusals, the `401` mapping, decimal
serialization, broker error fields, and cancel response parsing.
