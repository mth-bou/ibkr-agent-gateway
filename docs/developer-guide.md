# Developer Guide

This guide is the shortest path for developers who want to use or extend
`ibkr-agent-gateway`.

## Install and Run

From this checkout:

```bash
cargo run --bin ibkr-agent -- health --json
cargo run --bin ibkr-agent -- accounts list --json
```

Install the CLI locally:

```bash
cargo install --path .
ibkr-agent health --json
```

Embed the library from another local Rust project:

```bash
cargo add ibkr-agent-gateway --path /path/to/ibkr-agent-gateway
```

```rust
use ibkr_agent_gateway::prelude::*;

#[tokio::main]
async fn main() -> Result<(), GatewayError> {
    let gateway = Gateway::new(GatewayConfig::fake_local())?;
    let accounts = gateway.list_accounts().await?;
    println!("visible_accounts={}", accounts.len());
    Ok(())
}
```

## Backend Modes

Use `GatewayConfig::fake_local()` while developing. It reads deterministic
fixtures from `tests/fixtures/cpapi/` and needs no broker session.

Use `GatewayConfig::client_portal(url)` only when the Interactive Brokers
Client Portal Gateway is already running and manually authenticated outside this
project. TLS verification can be disabled only for localhost URLs.

For production-like deployments, read
[production-readiness.md](production-readiness.md). The CLI runner defaults to
fake fixtures only when `--config` is omitted. A real broker CLI run should pass
an explicit YAML config, such as `config/local.example.yaml`, and supply the
configured audit HMAC secret through the environment.

## CLI Surface

Read and session inspection:

```bash
ibkr-agent backend status --json
ibkr-agent session requirements --json
ibkr-agent account summary --account DU1234567 --json
ibkr-agent portfolio snapshot --account DU1234567 --json
ibkr-agent positions list --account DU1234567 --json
ibkr-agent market snapshot --contract-id 265598 --json
ibkr-agent executions list --account DU1234567 --json
```

Order preview is non-executable and must be explicitly enabled:

```bash
ibkr-agent orders preview \
  --account DU1234567 \
  --symbol AAPL \
  --side buy \
  --quantity 1 \
  --limit-price 100 \
  --currency USD \
  --enable-preview \
  --json
```

Paper submit/cancel commands require explicit paper enablement and an
idempotency key. Submit also requires the approval id returned by
`approvals create`:

```bash
ibkr-agent approvals create --account DU1234567 --ttl-seconds 300 --json
ibkr-agent orders submit --account DU1234567 --approval-id <approval_id> --idempotency-key paper-submit-001 --enable-paper --json
ibkr-agent orders cancel --account DU1234567 --broker-order-id paper-order-local --idempotency-key paper-cancel-001 --enable-paper --json
```

Live commands are gated local candidates unless a future broker adapter returns
broker-generated ids:

```bash
ibkr-agent orders live-submit \
  --account DU1234567 \
  --idempotency-key live-submit-001 \
  --enable-live \
  --live-scope \
  --open-kill-switch \
  --acknowledge-paper-to-live \
  --json
```

Audit review:

```bash
ibkr-agent audit tail --limit 20 --json
ibkr-agent audit export --limit 500 --json
```

## MCP

Local stdio MCP:

```bash
ibkr-agent mcp serve --transport stdio --describe --json
ibkr-agent mcp serve --transport stdio --json
```

`--describe` exits after a smoke-check description. Without it, the command
runs the stdio JSON-RPC loop, advertises only tools enabled by local scopes, and
audits every tool call.

Remote HTTP MCP is disabled by default. Enabling it requires complete
`RemoteMcpConfig`, an OAuth/OIDC issuer, RS256/RSA JWKS validation, a token-id
HMAC secret, accepted audiences, allowed scopes, and the independent safety
flag. See [remote-mcp-oauth.md](remote-mcp-oauth.md).

Example client configs live under `examples/mcp-clients/`.

## Validation

Run these before changing package behavior:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
```

Run packaging checks before release work:

```bash
cargo package --allow-dirty --no-verify --list
cargo publish --dry-run --locked
```

## Safety Rules for Contributors

- Do not return broker cookies, bearer tokens, credentials, raw headers, local
  paths, or raw session material from any CLI, MCP, log, fixture, or audit path.
- Keep broker logic provider-neutral; provider-specific behavior belongs in
  compatibility examples and tests.
- Keep write-capable flows fail-closed unless explicit config, scope, approval,
  idempotency, audit, and risk gates are satisfied.
- Preserve the public facade boundary in `src/public/*` and `src/lib.rs`; keep
  implementation details under `src/internal/*`.
