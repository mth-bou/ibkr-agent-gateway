# Public API Direction

`ibkr-agent-gateway` is intended to become a single user-facing Rust package with
two supported entrypoints:

- a CLI binary installed with `cargo install ibkr-agent-gateway`;
- an SDK-style library added with `cargo add ibkr-agent-gateway`.

The package is unofficial and is not affiliated with Interactive Brokers.

## Target CLI Entry Point

```bash
cargo install ibkr-agent-gateway
ibkr-agent health --json
ibkr-agent accounts list --json
ibkr-agent mcp serve --transport stdio --json
```

The CLI remains the operator-focused surface for local runs, smoke checks,
read-only account inspection, MCP server startup, audit tail/export, and gated
order workflows.

## Target SDK Entry Point

Developers embedding the gateway should use only the package facade:

```rust
use ibkr_agent_gateway::prelude::*;
use ibkr_agent_gateway::{Gateway, GatewayConfig};

#[tokio::main]
async fn main() -> Result<(), GatewayError> {
let config = GatewayConfig::fake_local();
let gateway = Gateway::new(config)?;

let session = gateway.session_status().await?;
let accounts = gateway.list_accounts().await?;
println!("session={:?} accounts={}", session.status, accounts.len());

    Ok(())
}
```

This API is the boundary that should stay understandable for downstream users.
Implementation modules may move during the packaging refactor without becoming
part of the stable public contract.

## Public Modules

### `Gateway`

The high-level service facade for embedded usage.

Target responsibilities:

- create a gateway from validated configuration;
- expose read-only broker/account/market data methods;
- expose MCP tool registry and execution helpers;
- route audit recording through configured storage;
- expose paper/live order workflows only through explicit safety gates.

### `GatewayConfig`

The public configuration input for embedded usage.

Target responsibilities:

- provide safe constructors such as `fake_local()`;
- load and validate runtime configuration;
- keep remote MCP, sidecar, paper, and live settings fail-closed by default;
- avoid leaking broker secrets or local session state into public output.

### `prelude`

Convenience exports for application code.

Target contents:

- `Gateway`;
- `GatewayConfig`;
- `GatewayError`;
- common typed identifiers and read-only result types that users need to call
  gateway methods.

### `mcp`

Embedding helpers for MCP usage.

Target responsibilities:

- list tool schemas;
- serve local stdio MCP;
- serve remote HTTP MCP only with validated OAuth/OIDC configuration;
- expose provider compatibility metadata without provider-specific core logic.

### `audit`

Audit helpers for users that embed the gateway.

Target responsibilities:

- configure audit storage;
- tail/export audit records;
- expose redacted audit event types, never raw tokens, cookies, credentials, or
  local session paths.

### `orders`

Order preview, paper, and live workflow facade.

Target responsibilities:

- expose preview and risk checks;
- enforce paper approval and idempotency;
- keep live trading behind feature, scope, approval, risk, kill-switch, audit,
  and migration gates.

## Compatibility Rules During `0.1.x`

- Only facade modules documented here are intended for downstream use.
- Implementation modules are allowed to move while packaging is being collapsed
  into one crate.
- Safety behavior must remain stable: read-only by default, fail-closed config,
  no secret output, paper before live, and live trading gated.
- Examples must use `ibkr_agent_gateway::*` imports only.
- `examples/embed_gateway.rs` and `examples/run_mcp_stdio.rs` are the
  compile-checked examples for the public SDK boundary.

## Release Gates

Before removing `publish = false`, the package must pass:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
cargo package --allow-dirty --no-verify --list
cargo publish --dry-run --locked
```

The package tarball must not include local agent directories, editor state,
build outputs, private config, broker session files, tokens, or machine-specific
paths.
