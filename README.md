# IBKR Agent Gateway

`ibkr-agent-gateway` is an unofficial Rust CLI, SDK, and MCP gateway for
Interactive Brokers workflows.

It is designed for local-first broker access through the Interactive Brokers
Client Portal Gateway, with typed domain models, provider-neutral MCP tools,
explicit scopes, deterministic risk gates, idempotency, and redacted audit
records.

This project is not affiliated with, endorsed by, or supported by Interactive
Brokers.

## What Works Today

The package is a single Cargo crate with two supported entrypoints:

- `ibkr-agent`: an operator/developer CLI;
- `ibkr_agent_gateway`: an embeddable Rust library facade.

Implemented surfaces include:

- offline fake backend fixtures for fast development and CI;
- local Client Portal Gateway read calls for session, accounts, portfolio,
  positions, contracts, market data, read-only orders, and executions;
- local MCP stdio serving, read-only tool discovery, scope enforcement, and
  tool-call audit;
- remote MCP HTTP authorization primitives with OAuth/OIDC, RS256 JWKS
  validation, protected-resource metadata, generic auth denials, and rate
  limiting;
- order preview and deterministic risk checks;
- paper submit/cancel lifecycle gates with approval and idempotency;
- live submit/cancel safety gates, kill switch, limits, and paper-to-live
  checklist checks;
- pluggable live order writer with a bundled Client Portal Gateway
  implementation that returns broker-generated order ids and handles the
  IBKR reply-chain confirmation protocol.

The CLI ships wired to a local-candidate writer so `orders submit
--enable-live` exercises the full gate stack offline. Operational
deployments wire `ClientPortalLiveWriter` against a configured Client
Portal Gateway — see [docs/production-readiness.md](docs/production-readiness.md).

## Safety Model

Defaults are deliberately conservative:

- broker credentials, cookies, bearer tokens, raw headers, and local session
  material must never be returned to agents, CLI output, logs, fixtures, or
  audit payloads;
- fake backend is available for offline development;
- remote MCP, sidecar relay, paper trading, and live trading all require
  explicit enablement;
- order preview is non-executable;
- paper workflows require approval and idempotency;
- live workflows require scope, config, approval, risk, kill switch, audit, and
  paper-to-live migration gates.

## Quick Start

From a local checkout:

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

Use the SDK from another local Rust project:

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

```bash
ibkr-agent backend status --json
ibkr-agent session requirements --json
ibkr-agent account summary --account DU1234567 --json
ibkr-agent positions list --account DU1234567 --json
ibkr-agent contracts resolve AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent market snapshot --contract-id 265598 --json
ibkr-agent orders preview --account DU1234567 --symbol AAPL --side buy --quantity 1 --limit-price 100 --enable-preview --json
ibkr-agent approvals create --account DU1234567 --preview-id <preview_id> --ttl-seconds 300 --json
ibkr-agent orders submit --account DU1234567 --approval-id <approval_id> --idempotency-key paper-submit-001 --enable-paper --json
ibkr-agent audit tail --limit 20 --json
```

## Developer Checks

Use the repo-native gates before changing public behavior:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
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

The published package documentation above is the developer-facing source for
current behavior.
