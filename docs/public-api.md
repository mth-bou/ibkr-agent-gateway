# Public API

The public Rust API is the package facade exposed by `src/lib.rs`. Downstream
code should import from `ibkr_agent_gateway::*`, not from `src/internal/*`.

The package exposes:

- `Gateway` and `GatewayConfig` at the crate root;
- `prelude` for common read-only domain types;
- `audit`, `config`, `mcp`, and `orders` facade modules.

Implementation modules under `src/internal/*` are not part of the stable public
contract.

## Gateway

`Gateway` is the high-level embeddable read facade. It currently supports:

- session status;
- keepalive;
- account listing;
- contract search.

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

## GatewayConfig

Use safe constructors instead of manually assembling internals:

```rust
use ibkr_agent_gateway::{Gateway, GatewayConfig};
use url::Url;

# fn build_fake() -> Result<Gateway, ibkr_agent_gateway::prelude::GatewayError> {
let fake = Gateway::new(GatewayConfig::fake_local())?;
# Ok(fake)
# }

# fn build_cp() -> Result<Gateway, Box<dyn std::error::Error>> {
let url = Url::parse("https://localhost:5000/v1/api")?;
let cpapi = Gateway::new(GatewayConfig::client_portal(url).with_verify_tls(false))?;
# Ok(cpapi)
# }
```

`with_verify_tls(false)` is valid only for local Client Portal Gateway URLs.
Account audit hashes use an ephemeral HMAC key by default; use
`with_audit_hmac_key` when a stable per-deployment audit correlation key is
required.

## Facade Modules

### `prelude`

Common imports for application code:

- `Gateway`, `GatewayConfig`;
- `GatewayError`, `ErrorCode`;
- account, session, contract, market, money, order preview, and read-only order
  domain types.

### `mcp`

Embedding helpers for MCP usage:

- tool schema lists via `broker_tool_schemas()` and
  `broker_tool_schemas_ref()`;
- explicit local tool schema lists via `local_tool_schemas()` and
  `local_tool_schemas_for_scopes(&scopes)`;
- compatibility live discovery via `broker_tool_schemas_with_live(true)`;
- local transport descriptions;
- scope guard helpers and forbidden generic write-tool refusals.

The MCP schema facade includes the mature local tool surface: consultative
reads, safety visibility, advanced market reads, bracket workflows, contextual
data reads, explicit paper/live write tools, and MCP approval creation. The
domain DTOs for those internal MCP handlers are intentionally not all re-exported
through the stable public facade in `0.3.x`; consumers should treat the schema
helpers and CLI/MCP transports as the compatibility boundary.

Remote HTTP MCP serving is available through the CLI and uses internal
authorization, protected-resource metadata, and scope-filtered JSON-RPC
routing. Production remote MCP requires OAuth/OIDC configuration and the
independent safety flag.

### `audit`

Audit facade exports:

- audit event and result models;
- SQLite writer and tail/export/verify DTOs;
- redaction helpers and replay helpers.

Audit output must remain redacted: no bearer tokens, cookies, credentials, raw
headers, local secret paths, raw account ids, or broker session material.

### `orders`

Order/risk facade exports:

- order intent, preview, validated order, and read-only order models. Validated
  orders carry resolved symbol and asset-class metadata for later live gates;
- deterministic risk policy and refusal types;
- paper submit/cancel lifecycle functions with approval and idempotency;
- paper and live modify types remain internal to the MCP/CLI maturity surface in
  `0.3.x`;
- live submit/cancel gate functions with limits, kill switch, audit, and
  paper-to-live checklist checks;
- `LivePolicyRegistry` and `StaticPolicyRegistry` for server-side live policy
  lookup;
- `apply_live_rate_counters` for deriving live submit counters and session
  notional from durable audit workflow state before evaluating limits;
- `LiveOrderWriter` trait and bundled `LocalCandidateLiveWriter` /
  `RefusingLiveWriter` implementations so live submit/cancel return
  writer-provided broker order ids.

Live submit and cancel enforce the gate stack and then delegate the broker
call to a `LiveOrderWriter` chosen at deployment time. The bundled
`ClientPortalLiveWriter` is the production adapter against the Interactive
Brokers Client Portal Gateway; consumers may also implement their own
adapter for alternative backends.

### `config`

Configuration facade exports typed config structures:

- runtime gateway config;
- audit retention/storage config;
- remote MCP config;
- sidecar config;
- preview, paper, and live trading config;
- safety flags.

The public config module intentionally does not re-export every internal
`validate_*` helper. Use the typed configuration methods and gateway
constructors as the stable boundary.

## Compatibility Rules for `0.3.x`

- Public consumers should use only crate-root exports and facade modules.
- Internal module layout may change without a semver guarantee.
- Safety behavior must remain stable: fail-closed defaults, redacted output,
  scoped access, idempotency for write-capable flows, and gated live trading.
- Examples must use `ibkr_agent_gateway::*` imports only.

## Examples

Compile-checked examples:

- `examples/embed_gateway.rs`
- `examples/run_mcp_stdio.rs`

Provider client examples:

- `examples/mcp-clients/generic-inspector.json`
- `examples/mcp-clients/cursor.json`
- `examples/mcp-clients/continue.json`
