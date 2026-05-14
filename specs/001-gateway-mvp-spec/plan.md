# Implementation Plan: IBKR Agent Gateway Read-Only MVP

**Branch**: `001-gateway-mvp-spec` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-gateway-mvp-spec/spec.md`

## Summary

Build the first local, read-only IBKR Agent Gateway increment: a Rust workspace
with deterministic domain types, a Client Portal Gateway read backend, a local
CLI, local MCP read-only tools, structured errors, scoped access checks, audit
events, a separate runtime configuration layer, and offline fixture testing.
The feature deliberately excludes order preview, submit, cancel, remote public
MCP, sidecar relay, and live trading writes.

## Technical Context

**Language/Version**: Rust stable 1.94.1 available locally, Rust 2024 edition.
Pin the toolchain in the repo so CI and local development use the same compiler.

**Primary Dependencies**: `tokio`, `reqwest`, `serde`, `serde_json`,
`thiserror`, `tracing`, `tracing-subscriber`, `clap`, `config`, `rust_decimal`,
`time`, `uuid`, `url`, `secrecy`, `zeroize`, `sqlx` with SQLite, `async-trait`,
`schemars`, `rmcp`, and `wiremock` for offline broker tests.

**Storage**: SQLite for local append-only audit events and read-only call
correlation. Broker data is read through the backend and not cached as an MVP
source of truth.

**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets`,
`cargo test --workspace`, offline integration tests with a fake Client Portal
Gateway, MCP tool/schema tests, audit redaction tests, and fixture replay tests.

**Target Platform**: Local developer/user machine on Linux first. The gateway
binds only to local interfaces for this feature and talks to a local IBKR Client
Portal Gateway session.

**Project Type**: Rust Cargo workspace containing domain libraries, broker
adapter libraries, runtime configuration, audit/scope libraries, a CLI binary,
and a local MCP server binary/library.

**Performance Goals**: Gateway overhead under 500 ms p95 for local read-only
tool calls excluding broker latency; audit writes under 50 ms p95 locally;
offline fixture test suite completes in under 30 seconds.

**Constraints**: Read-only only; no order preview/submit/cancel tools; fail
closed on missing account, missing scope, ambiguous contract, stale market data,
unavailable backend, or unsafe output; no secrets in logs, MCP responses, audit
records, snapshots, or user-visible errors.

**Scale/Scope**: Single-user local MVP with one active local broker backend and
the read-only tools from P0/P1 of the project plan: health/session, accounts,
account summary, positions, portfolio snapshot, contract search/resolve, market
snapshot, historical bars if supported by fixtures, order list/status/executions,
and audit read.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **MCP boundary**: PASS. Broker capabilities are exposed through typed MCP
  tools and CLI/admin surfaces. LLM clients do not talk directly to IBKR, do not
  hold IBKR secrets, and do not control core broker logic.
- **Deterministic finance domain**: PASS. Domain entities are represented with
  typed identifiers, money, quantity, account, position, contract, market-data,
  read-only order, and typed error models. Free text is not executable trading
  input.
- **Read-only/write posture**: PASS. This feature is read-only. Preview, submit,
  cancel, paper-write, live-write, remote relay, and sidecar behavior are out of
  scope. Any write-like request fails closed and is audited.
- **Auth and scopes**: PASS. The plan defines minimal read scopes for all local
  MCP tools. Local scope enforcement is separated from IBKR Client Portal
  Gateway session authentication.
- **Risk and approval**: PASS. Order risk and approval flows are not implemented
  in this feature. The future order boundary is preserved by refusing order
  preview, submit, cancel, prompt-to-trade, and ambiguous broker actions.
- **Audit and observability**: PASS. Significant tool calls, denials, failures,
  completions, and backend session changes emit audit events with redaction and
  correlation. Logs and errors use typed codes and avoid secrets.
- **Testing evidence**: PASS. Required test classes are unit tests, offline
  integration tests, MCP schema/tool-list tests, audit redaction tests, scope
  denial tests, and fixture replay tests.

## Project Structure

### Documentation (this feature)

```text
specs/001-gateway-mvp-spec/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── audit-events.md
│   ├── cli.md
│   ├── config.md
│   └── mcp-tools.md
└── checklists/
    └── requirements.md
```

### Source Code (repository root)

```text
Cargo.toml
rust-toolchain.toml
crates/
├── ibkr-domain/
│   └── src/
├── ibkr-cpapi/
│   └── src/
├── ibkr-backend/
│   └── src/
├── ibkr-config/
│   └── src/
├── ibkr-audit/
│   └── src/
├── ibkr-auth/
│   └── src/
├── ibkr-mcp/
│   └── src/
└── ibkr-cli/
    └── src/
docs/
├── getting-started-local.md
├── ibkr-client-portal-gateway.md
├── mcp-local.md
├── tools.md
├── scopes.md
├── audit-log.md
└── testing.md
tests/
├── fixtures/
├── contract-tests/
├── integration/
└── replay/
```

**Structure Decision**: Use a Cargo workspace from the start because the
constitution requires separated domain, broker adapter, MCP, auth/scope, audit,
configuration, and CLI responsibilities. `ibkr-domain` stays free of HTTP, MCP,
OAuth, LLM, storage, and runtime configuration ownership. `ibkr-config` owns
local configuration loading and read-only safety validation. `ibkr-auth` is
intentionally limited to local identity and scope enforcement for this feature;
full OAuth/OIDC front-door behavior is a later feature.

## Complexity Tracking

No constitution violations.
