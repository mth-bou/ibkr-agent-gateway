# Implementation Plan: IBKR Agent Gateway Read-Only MVP

**Branch**: `001-gateway-mvp-spec` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-gateway-mvp-spec/spec.md`, aligned with `/specs/000-project-roadmap/`.

## Summary

Build the first local, read-only IBKR Agent Gateway increment: a Rust workspace with deterministic domain types, a Client Portal Gateway read backend, fake backend fixtures, a local CLI, local MCP stdio read-only tools, local scope checks, structured errors, explicit market-data status, keepalive/tickle handling, HMAC-based audit redaction, a separate runtime configuration layer, and Cargo-discoverable offline tests.

The feature deliberately excludes order preview, submit, cancel, modify, approve, paper writes, live writes, remote public MCP, sidecar relay, direct broker OAuth2, provider-specific LLM clients, and live trading.

## Technical Context

**Language/Version**: Rust stable, Rust 2024 edition. Pin the selected local toolchain in `rust-toolchain.toml` during repository bootstrap.

**Primary Dependencies**: `tokio`, `reqwest`, `serde`, `serde_json`, `thiserror`, `tracing`, `tracing-subscriber`, `clap`, `config`, `rust_decimal`, `time`, `uuid`, `url`, `secrecy`, `zeroize`, `hmac`, `sha2`, `sqlx` with SQLite, `async-trait`, `schemars`, `rmcp`, and `wiremock` for offline broker tests.

**Storage**: SQLite for local append-only audit events and read-only call correlation. Broker data is read through the backend and is not cached as an MVP source of truth.

**Testing**: `cargo fmt --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`, offline integration tests with a fake Client Portal Gateway, MCP tool/schema tests, audit redaction tests, scope denial tests, secret scans, and fixture replay tests. Top-level Rust test files must be Cargo-discoverable or included through explicit test harnesses.

**Target Platform**: Local developer/user machine on Linux first. The gateway binds only to local interfaces for this feature and talks to a local IBKR Client Portal Gateway session or fake backend.

**Project Type**: Rust Cargo workspace containing domain libraries, broker adapter libraries, runtime configuration, audit/scope libraries, a CLI binary, and a local MCP server binary/library.

**Performance Goals**: Gateway overhead under 500 ms p95 for local read-only tool calls excluding broker latency; audit writes under 50 ms p95 locally; offline fixture test suite completes in under 30 seconds.

**Constraints**: Read-only only; no order preview/submit/cancel/modify/approve tools; fail closed on missing account, missing scope, ambiguous contract, unsupported asset class, stale market data according to policy, unavailable backend, unavailable/expired broker session, unsafe output, or invalid configuration; no secrets in logs, MCP responses, audit records, snapshots, fixtures, or user-visible errors.

**Scale/Scope**: Single-user local MVP with one active local broker backend and the read-only tools from the project plan: health/session, accounts, account summary, positions, portfolio snapshot, contract search/resolve for stock/ETF, market snapshot, historical bars if supported by fixtures/backend, order list/status/executions, and audit read.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **MCP boundary**: PASS. Broker capabilities are exposed through typed MCP tools and CLI/admin surfaces. LLM clients do not talk directly to IBKR, do not hold IBKR secrets, and do not control core broker logic.
- **Deterministic finance domain**: PASS. Domain entities are represented with typed identifiers, money, quantity, account, position, contract, market-data, read-only order, and typed error models. Free text is not executable trading input.
- **Read-only/write posture**: PASS. This feature is read-only. Preview, submit, cancel, modify, approve, paper-write, live-write, remote relay, and sidecar behavior are out of scope. Any write-like request fails closed and is audited.
- **Auth and scopes**: PASS. The plan defines minimal local read scopes for all local MCP tools. Local scope enforcement is separated from future OAuth/OIDC and from IBKR Client Portal Gateway session authentication.
- **Risk and approval**: PASS. Order risk and approval flows are not implemented in this feature. The future order boundary is preserved by refusing order preview, submit, cancel, modify, approve, prompt-to-trade, and ambiguous broker actions.
- **Audit and observability**: PASS. Audit recorder, redaction, HMAC account hashing, SQLite persistence, and record-only events are foundational. Audit tail is a user story after operations emit events.
- **Testing evidence**: PASS. Required test classes are unit tests, offline integration tests, MCP schema/tool-list tests, audit redaction tests, scope denial tests, keepalive/session tests, contract tests, and fixture replay tests.

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
│   ├── cpapi-endpoints.md
│   ├── error-codes.md
│   ├── market-data-policy.md
│   ├── mcp-tools.md
│   └── scopes.md
└── checklists/
    ├── requirements.md
    └── safety.md
```

### Source Code (repository root)

```text
Cargo.toml
rust-toolchain.toml
.cargo/config.toml
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
│   ├── migrations/
│   └── src/
├── ibkr-auth/
│   └── src/
├── ibkr-mcp/
│   └── src/
└── ibkr-cli/
    └── src/
config/
└── local.example.yaml
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
│   └── cpapi/
├── contract_cli_us1.rs
├── contract_cli_us2.rs
├── contract_mcp_broker_tool_list.rs
├── contract_mcp_schemas.rs
├── contract_audit_events.rs
├── contract_cli_audit_tail.rs
├── contract_mcp_audit_tail.rs
├── integration_backend_status.rs
├── integration_accounts_list.rs
├── integration_keepalive.rs
├── integration_account_context_refusals.rs
├── integration_portfolio_positions.rs
├── integration_contracts_market.rs
├── integration_orders_readonly.rs
├── integration_write_refusals.rs
├── integration_mcp_scope_denials.rs
├── integration_mcp_redaction.rs
├── integration_audit_sqlite.rs
├── integration_quickstart_readonly.rs
├── integration_performance_budgets.rs
├── replay_audit_redaction.rs
└── replay_secret_scan.rs
```

**Structure Decision**: Use a Cargo workspace from the start because the constitution requires separated domain, broker adapter, MCP, auth/scope, audit, configuration, and CLI responsibilities. `ibkr-domain` stays free of HTTP, MCP, OAuth, LLM, storage, and runtime configuration ownership. `ibkr-config` owns local configuration loading and read-only safety validation. `ibkr-auth` is intentionally limited to local identity and scope enforcement for this feature; full OAuth/OIDC front-door behavior is a later feature.

## Complexity Tracking

No constitution violations. The only intentional complexity is building audit and fake-backend infrastructure early so later MCP, risk, and order features do not require retrofitting safety-critical foundations.
