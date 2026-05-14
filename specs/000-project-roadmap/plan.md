# Implementation Plan: IBKR Agent Gateway Complete Roadmap

**Branch**: `000-project-roadmap` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

**Input**: Complete product roadmap for a Rust-first MCP/OAuth gateway for Interactive Brokers.

## Summary

Build `ibkr-agent-gateway`: a provider-neutral Rust gateway that exposes Interactive Brokers capabilities through typed MCP tools, CLI/operator commands, scoped authorization, redacted audit, deterministic risk checks, and staged order capabilities.

The project is intentionally staged:

1. local read-only Client Portal Gateway MVP;
2. order preview and deterministic risk engine;
3. paper trading submission/cancel with explicit approval;
4. remote MCP over OAuth/OIDC;
5. local sidecar relay for retail Client Portal Gateway sessions;
6. provider compatibility with OpenAI, Anthropic, Cursor, Continue, and MCP inspectors;
7. live trading behind strict gates.

## Technical Context

**Language/Version**: Rust stable, Rust 2024 edition. Pin the selected toolchain in `rust-toolchain.toml` during repository bootstrap rather than hard-coding a guessed compiler version in the spec.

**Primary Dependencies**: `tokio`, `reqwest`, `serde`, `serde_json`, `thiserror`, `tracing`, `clap`, `config`, `rust_decimal`, `time`, `uuid`, `url`, `secrecy`, `zeroize`, `hmac`, `sha2`, `sqlx` with SQLite, `async-trait`, `schemars`, `rmcp`, `wiremock`, and OAuth/JWT crates selected during the remote MCP feature.

**Storage**: SQLite for local audit in early phases. Remote/multi-user deployments may add Postgres later behind an audit storage trait. Broker data is not cached as source of truth in early phases.

**Testing**: Unit tests, Cargo-discovered integration tests, contract tests, MCP tool/schema snapshots, OAuth token validation tests, audit/redaction replay tests, fake broker fixtures, provider compatibility tests, and paper/live smoke tests gated by explicit environment flags.

**Target Platforms**: Linux first for local development and CI; macOS/Windows can be added after local MVP if sidecar packaging requires them. Remote gateway targets containerized Linux.

**Project Type**: Rust Cargo workspace with multiple crates and one or more binaries.

**Performance Goals**: Local read-only gateway overhead under 500 ms p95 excluding broker latency; audit write p95 under 50 ms locally; order preview/risk checks under 250 ms p95 excluding broker preview calls; remote auth checks under 50 ms p95 excluding JWKS refresh.

**Constraints**: No copied IBKR official source code; no broker secrets returned to agents; fail closed on ambiguity or missing authorization; no live trading until the live feature spec; no provider-specific coupling in core crates.

## Architecture

```text
MCP Client / CLI / Provider Connector
        │
        ▼
ibkr-mcp / ibkr-cli
        │
        ▼
Auth + Scope Guard ── Audit Recorder
        │
        ▼
Domain Services
        ├── Read Services
        ├── Risk Engine
        ├── Approval Service
        └── Order Service
        │
        ▼
IbkrBackend trait
        ├── Client Portal Gateway backend
        ├── Fake backend
        ├── Future IBKR OAuth2 Web API backend
        └── Optional future TWS/IB Gateway backend
```

## Repository Structure

```text
Cargo.toml
rust-toolchain.toml
crates/
├── ibkr-domain/              # pure identifiers, money, contracts, accounts, market data, orders, errors
├── ibkr-cpapi/               # Client Portal Gateway HTTP adapter
├── ibkr-backend/             # backend trait, fake backend, backend factory
├── ibkr-config/              # runtime config loading/validation
├── ibkr-auth/                # local scopes, future OAuth/OIDC validation facade
├── ibkr-audit/               # audit models, HMAC redaction, append-only storage
├── ibkr-mcp/                 # MCP tools, schemas, stdio/http transports
├── ibkr-cli/                 # operator CLI
├── ibkr-risk/                # future deterministic order risk policies
├── ibkr-orders/              # future order preview/submit/cancel lifecycle
├── ibkr-approval/            # future approval workflow
├── ibkr-oauth/               # future remote MCP OAuth/OIDC front-door
├── ibkr-sidecar/             # future local sidecar relay client/server
└── ibkr-provider-compat/     # future provider compatibility tests/helpers
docs/
specs/
└── 000-project-roadmap/
```

## Feature Specs

| Spec | Feature | Purpose | Write Capability |
|------|---------|---------|------------------|
| `001-gateway-mvp-spec` | Local read-only MVP | CP Gateway read backend, CLI, local MCP, scopes, audit, fixtures | None |
| `002-order-preview-risk` | Order preview + risk | `OrderIntent`, risk policy, preview, no submit | Preview only |
| `003-paper-submit-approval` | Paper submit/cancel | approval, idempotency, lifecycle, paper only | Paper only |
| `004-remote-mcp-oauth` | Remote MCP OAuth/OIDC | HTTP MCP, JWT/OIDC, scopes, audience, issuer | Same as enabled features |
| `005-sidecar-relay` | Local sidecar relay | bridge remote MCP to local CP Gateway | Same as enabled features |
| `006-provider-compatibility` | Provider tests | OpenAI/Anthropic/Cursor/Continue compatibility | None new |
| `007-live-trading-gated` | Live trading | live gates, limits, kill switch, audit retention | Live gated |
| `008-operations-hardening` | Observability/replay/export | monitoring, audit export, runbooks, packaging | None new |

## Topologies

### Local Read-Only

```text
MCP client or CLI -> local gateway -> local Client Portal Gateway -> IBKR
```

### Remote OAuth MCP With Direct Broker OAuth2

```text
MCP client -> remote gateway OAuth/OIDC -> IBKR OAuth2 backend
```

### Remote OAuth MCP With Local Sidecar Relay

```text
MCP client -> remote gateway OAuth/OIDC -> sidecar relay -> local sidecar -> local Client Portal Gateway -> IBKR
```

## Constitution Check

- **Provider-neutral MCP**: PASS. MCP is the agent boundary; OpenAI/Anthropic are compatibility targets.
- **Deterministic finance core**: PASS. Broker and order actions use typed inputs and deterministic services.
- **Read-only first**: PASS. The first spec exposes no write tool.
- **Separated auth**: PASS. MCP authorization is separate from IBKR broker authentication.
- **Audit first**: PASS. Audit is present before remote MCP and before write features.
- **Risk before submit**: PASS. Preview/risk is a separate spec before paper submit.
- **Paper before live**: PASS. Paper submission is a separate spec before live trading.
- **Sidecar realism**: PASS. Remote retail usage accounts for local/manual Client Portal Gateway authentication.

## Complexity Tracking

No constitution violations. The major complexity is intentionally deferred into separate specs rather than merged into the MVP.
