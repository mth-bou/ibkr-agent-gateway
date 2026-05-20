# IBKR Agent Gateway

[![crates.io](https://img.shields.io/crates/v/ibkr-agent-gateway.svg)](https://crates.io/crates/ibkr-agent-gateway)
[![docs.rs](https://img.shields.io/docsrs/ibkr-agent-gateway)](https://docs.rs/ibkr-agent-gateway)
[![CI](https://github.com/mth-bou/ibkr-agent-gateway/actions/workflows/ci.yml/badge.svg)](https://github.com/mth-bou/ibkr-agent-gateway/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

`ibkr-agent-gateway` is an unofficial Rust CLI, SDK, and MCP gateway for
Interactive Brokers workflows.

It gives agents and operator tools a controlled way to inspect IBKR accounts,
stage order workflows, and keep every sensitive boundary explicit: typed
requests, scoped access, deterministic risk gates, idempotency, and redacted
audit records.

This project is not affiliated with, endorsed by, or supported by Interactive
Brokers.

## Why This Exists

Use it when you want an agent, internal tool, or operator workflow to inspect an
IBKR account through CLI or MCP without exposing broker session material.

The gateway keeps broker access behind explicit scopes, typed requests, safe
output shapes, and append-only audit evidence. Order workflows are staged on
purpose: preview first, paper trading behind approval and idempotency, then live
trading only when an operator explicitly enables the full gate stack.

It is not a trading strategy engine, autonomous bot, or shortcut around IBKR
authentication. It is an audited control plane for broker operations.

## At a Glance

| Area | What you get |
|------|--------------|
| 🧰 CLI | `ibkr-agent` for local operator and developer workflows |
| 🦀 Rust SDK | `ibkr_agent_gateway` facade for embedding the gateway in Rust apps |
| 🔌 MCP | Provider-neutral MCP tools for local and remote agent clients |
| 🧪 Offline mode | Fake backend fixtures and local-candidate writers for CI and smoke tests |
| 🧾 Audit | Redacted, append-only evidence with account hashing and verification |
| 🛡️ Trading gates | Preview, approval, idempotency, risk checks, kill switch, and live limits |

## What Works Today

The package is a single Cargo crate with two supported entrypoints:

| Entrypoint | Purpose |
|------------|---------|
| `ibkr-agent` | Operator/developer CLI |
| `ibkr_agent_gateway` | Embeddable Rust library facade |

Implemented surfaces include:

- ✅ offline fake backend fixtures for fast development and CI;
- ✅ local Client Portal Gateway read calls for session, accounts, portfolio,
  positions, contracts, market data, read-only orders, and executions;
- ✅ local MCP stdio serving, scope-filtered tool discovery, consultative and
  contextual read tools, preview/paper/live-gated order tools, bracket tools,
  MCP approval creation, and tool-call audit;
- ✅ remote MCP HTTP serving for `POST /mcp` with OAuth/OIDC, RS256 JWKS
  validation, protected-resource metadata, generic auth denials, and JSON-RPC
  tool routing;
- ✅ order preview and deterministic risk checks;
- ✅ paper submit/cancel/modify lifecycle gates with approval and idempotency;
- ✅ live submit/cancel/modify and MCP bracket safety gates, kill switch,
  limits, server-side rate counters, lifecycle reconciliation, and
  paper-to-live checklist checks;
- ✅ pluggable live order writer with a bundled Client Portal Gateway
  implementation that returns broker-generated order ids and handles the
  IBKR reply-chain confirmation protocol.

The live CLI commands default to a local-candidate writer so the full gate
stack (kill switch, allowlist, approval, idempotency, paper-to-live) is
exercised end-to-end offline — see the live-submit row in
[Common CLI Commands](#common-cli-commands) for the complete invocation.
Use `--live-broker client-portal` with a Client Portal Gateway config to call
the bundled production writer — see
[docs/production-readiness.md](docs/production-readiness.md).

The MCP registry currently exposes 45 explicit tools when all scopes are
enabled. The full tool and scope matrix is documented in
[docs/mcp-local.md](docs/mcp-local.md) and [docs/scopes.md](docs/scopes.md).

## Safety Model 🛡️

Defaults are deliberately conservative:

- 🔒 broker credentials, cookies, bearer tokens, raw headers, and local session
  material must never be returned to agents, CLI output, logs, fixtures, or
  audit payloads;
- 🧪 fake backend is available for offline development;
- 🚪 remote MCP, sidecar relay, paper trading, and live trading all require
  explicit enablement;
- 👀 order preview is non-executable;
- 🧾 paper workflows require approval and idempotency;
- 🚨 live workflows require scope, config, approval, risk, kill switch, audit,
  durable idempotency, and paper-to-live migration gates.

## Quick Start 🚀

Smoke-test the local fake backend from a checkout:

```bash
cargo run --bin ibkr-agent -- health --json
cargo run --bin ibkr-agent -- accounts list --json
cargo run --bin ibkr-agent -- mcp serve --transport stdio --describe --json
```

Install the CLI from the checkout:

```bash
cargo install --path .
ibkr-agent health --json
```

Or install the binary from crates.io:

```bash
cargo install ibkr-agent-gateway
ibkr-agent health --json
```

Use the SDK from another Rust project:

```bash
cargo add ibkr-agent-gateway
```

Or, while developing from this checkout:

```bash
cargo add ibkr-agent-gateway --path /path/to/ibkr-agent-gateway
```

Minimal embedded usage:

```rust
use ibkr_agent_gateway::prelude::*;

#[tokio::main]
async fn main() -> Result<(), GatewayError> {
    let gateway = Gateway::new(GatewayConfig::fake_local())?;
    let session = gateway.session_status().await?;
    let accounts = gateway.list_accounts().await?;

    println!("session={:?} accounts={}", session.status, accounts.len());
    Ok(())
}
```

## Common CLI Commands

| Workflow | Command |
|----------|---------|
| Backend status | `ibkr-agent backend status --json` |
| Session requirements | `ibkr-agent session requirements --json` |
| Account summary | `ibkr-agent account summary --account DU1234567 --json` |
| Portfolio snapshot | `ibkr-agent portfolio snapshot --account DU1234567 --json` |
| Positions | `ibkr-agent positions list --account DU1234567 --json` |
| Contract resolution | `ibkr-agent contracts resolve AAPL --asset-class stock --currency USD --exchange SMART --json` |
| Market snapshot | `ibkr-agent market snapshot --contract-id 265598 --json` |
| Order preview | `ibkr-agent orders preview --account DU1234567 --symbol AAPL --side buy --quantity 1 --limit-price 100 --enable-preview --json` |
| Approval | `ibkr-agent approvals create --account DU1234567 --preview-id <preview_id> --ttl-seconds 300 --json` |
| Paper submit | `ibkr-agent orders submit --account DU1234567 --approval-id <approval_id> --idempotency-key paper-submit-001 --enable-paper --json` |
| Live-gated submit | `ibkr-agent orders live-submit --account DU1234567 --approval-id <approval_id> --idempotency-key live-submit-001 --enable-live --live-scope --open-kill-switch --acknowledge-paper-to-live --live-broker local-candidate --json` |
| Executions list | `ibkr-agent executions list --account DU1234567 --json` |
| MCP stdio | `ibkr-agent mcp serve --transport stdio --json` |
| MCP HTTP | `ibkr-agent --config config/remote.example.yaml mcp serve --transport http --enable-remote-mcp --bind 127.0.0.1:8080` |
| Sidecar relay accept | `ibkr-agent sidecar relay accept --remote-instance-id remote-1 --sidecar-id sidecar-example --tool-name ibkr_accounts_list --scope ibkr:accounts:read --payload-json '{}' --json` |
| Audit tail | `ibkr-agent audit tail --limit 20 --json` |
| Audit verification | `ibkr-agent audit verify --json` |

## Developer Checks ✅

Use the repo-native gates before changing public behavior:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
# Focused replay of secret-scan and redaction tests by name filter:
cargo test --workspace --features unstable-internal-test-support secret
```

Useful packaging checks:

```bash
cargo package --allow-dirty --no-verify --list
cargo publish --dry-run --locked
```

## Documentation

Start with [docs/README.md](docs/README.md). The main developer path is:

- [Developer Guide](docs/developer-guide.md)
- [Production Readiness](docs/production-readiness.md)
- [Public API](docs/public-api.md)
- [MCP](docs/mcp-local.md)
- [Scopes](docs/scopes.md)
- [Audit Log](docs/audit-log.md)
- [Testing](docs/testing.md)
- [Changelog](CHANGELOG.md)

The linked docs above are the developer-facing source of truth for current
behavior; this README is a high-level entry point.

## Contributing 🤝

- [Contribution Guide](CONTRIBUTING.md): workflow, branching, versioning, and
  safety areas requiring code-owner review.
- [Security Policy](SECURITY.md): report vulnerabilities through GitHub
  Security Advisories — do not file public issues for credential exposure,
  live-gate bypasses, audit-redaction flaws, or OAuth validation issues.
- Code owners are listed in [.github/CODEOWNERS](.github/CODEOWNERS); changes
  in `src/internal/orders/`, `src/internal/audit/`, `src/internal/cpapi/`, and
  `src/internal/oauth/` require explicit review.
