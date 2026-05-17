# Production Readiness

This checklist is for operators and developers preparing a real deployment of
`ibkr-agent-gateway`.

The package is intentionally conservative, but it is still unofficial software
for financial workflows. Treat production enablement as an explicit deployment
decision, not as a default mode.

## Current Deployment Status

Ready for production-like validation:

- SDK facade with fake and Client Portal Gateway backend constructors;
- CLI runtime config loading for fake or Client Portal Gateway backends;
- read-only broker data paths;
- redacted audit storage and export;
- local MCP stdio serving with scope-filtered tool discovery and audited calls;
- remote MCP OAuth/OIDC validation primitives;
- preview, paper, sidecar, provider compatibility, and live-gate domain logic;
- live order writer trait with a bundled Client Portal Gateway implementation
  that returns broker-generated order ids and handles the IBKR reply-chain
  confirmation protocol.

Live submit and cancel now delegate the broker call to a
[`LiveOrderWriter`](../src/internal/orders/live_writer.rs) implementation
chosen at deployment time:

- [`ClientPortalLiveWriter`](../src/internal/cpapi/live_writer.rs) for
  production deployments behind a real Client Portal Gateway;
- `LocalCandidateLiveWriter` for CLI smoke tests and offline development —
  it returns a deterministic local identifier and performs no network I/O;
- `RefusingLiveWriter` as a fail-closed default for environments that have
  not yet been validated.

The CLI `orders submit --enable-live` / `orders cancel --enable-live`
commands ship wired to `LocalCandidateLiveWriter` because the CLI is a
smoke harness for the gate stack, not a production live-trading entrypoint.
Operational deployments construct `ClientPortalLiveWriter` from a
configured `ClientPortalClient` and inject it into the live submit/cancel
flow before exposing live tools.

## Hard Prerequisites

Before exposing any non-local workflow:

- run the full validation suite:

  ```bash
  cargo fmt --check
  cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
  cargo test --workspace --features unstable-internal-test-support
  cargo doc --workspace --no-deps
  ```

- verify the exact binary/library artifact that will be deployed;
- configure audit storage and verify writes, tail reads, and exports;
- verify CLI `--config` loading with a missing-path negative test and a real
  config smoke test;
- supply stable deployment HMAC secrets from a secret manager;
- confirm no broker cookies, bearer tokens, credentials, raw headers, local
  paths, raw account ids, or Client Portal Gateway session material appear in
  CLI, MCP, logs, fixtures, or audit output;
- keep broker authentication separate from MCP/OAuth authorization.

## Required Secrets

Use deployment-specific secrets. Do not commit them.

| Secret | Purpose |
|--------|---------|
| `IBKR_AUDIT_HMAC_SECRET` | stable HMAC key for account/audit correlation |
| `IBKR_REMOTE_TOKEN_HMAC_SECRET` | HMAC key for remote token-id audit hashes |

Secrets should be at least 32 random bytes. Rotate only with a plan for audit
correlation discontinuity.

## Read-Only Broker Deployment

For real broker reads:

- run the Interactive Brokers Client Portal Gateway locally or in the intended
  private network boundary;
- complete IBKR authentication outside this gateway;
- use `GatewayConfig::client_portal(url)` or a verified runtime config path;
- allow `verify_tls=false` only for localhost URLs;
- test session required, expired, unavailable, and keepalive behavior before
  using account or market-data tools.

## Remote MCP

Remote MCP requires:

- `remote_mcp.enabled: true`;
- `safety.remote_public_mcp_enabled: true`;
- HTTPS issuer and JWKS URLs;
- RS256 JWTs against RSA JWKS keys in production builds;
- accepted audiences/resources;
- explicit allowed gateway scopes;
- `remote_mcp.token_id_hmac_secret`;
- rate limiting at the gateway plus upstream connection limits.

Remote MCP bearer tokens must never be forwarded to IBKR or stored raw in audit.

## Sidecar Relay

Use sidecar relay only when:

- remote MCP is already validated;
- a sidecar identity and pairing record exist;
- heartbeat and relay session binding are verified;
- forwarded payloads contain hashes and safe tool metadata only;
- local Client Portal Gateway login still happens manually outside the gateway.

## Preview and Paper Trading

Order preview is non-executable. It must remain useful for validation without
creating broker-side state.

Paper submit/cancel require:

- explicit paper enablement;
- paper scopes;
- persisted approval records;
- idempotency keys;
- audit availability;
- refusal tests for disabled config, missing approval, and idempotency
  conflicts.

## Live Trading

Do not enable live trading until all items in [paper-to-live.md](paper-to-live.md)
and [live-runbook.md](live-runbook.md) are satisfied.

Live submit/cancel must fail closed unless all gates pass:

- live config and independent safety flag;
- live account allowlist;
- live scope;
- approval and validated preview;
- idempotency;
- live risk limits;
- open kill switch;
- audit availability;
- paper-to-live checklist acknowledgement.

Close the kill switch on uncertainty.

### Live order writer wiring

When the gates pass, the live flow delegates the broker call to a
`LiveOrderWriter`. Production deployments wire `ClientPortalLiveWriter`
against a configured `ClientPortalClient`:

```rust
use ibkr_agent_gateway::testing::cpapi::{ClientPortalClient, ClientPortalLiveWriter};

let cp_client = ClientPortalClient::new(base_url, verify_tls)?;
let writer = ClientPortalLiveWriter::new(cp_client);
// ... inject `&writer` into submit_live_order / cancel_live_order ...
```

The bundled writer:

- posts orders to `/iserver/account/{accountId}/orders` with `cOID` set to
  the idempotency key for broker-side de-duplication;
- handles the reply chain (`POST /iserver/reply/{replyId}`) up to a
  configurable depth (default 5) so warning prompts are confirmed once;
- refuses market orders and missing limit prices at the writer boundary,
  in addition to the upstream risk gates;
- returns the broker-generated order id in the lifecycle record;
- maps `401` to `BROKER_SESSION_REQUIRED`, transport failures to
  `BROKER_BACKEND_UNAVAILABLE`, and oversized responses to
  `BROKER_RESPONSE_INVALID`.

Validate the writer in a paper environment before promoting to live. The
contract test suite
(`tests/contract_cpapi_live_writer.rs`) covers the happy path, reply
confirmation, depth limit, market/limit refusals, `401`, decimal
serialization, broker error fields, and cancel response parsing.

## Package Publication

### Pre-Publish Runbook

Run the full gate suite, confirm the tarball contents, then tag and publish:

```bash
# 1. Quality gates
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
cargo doc --workspace --no-deps
cargo audit

# 2. Tarball contents
cargo package --allow-dirty --no-verify --list
cargo publish --dry-run --locked

# 3. Tag and publish
git tag -a vX.Y.Z -m "vX.Y.Z — release notes"
git push origin vX.Y.Z
cargo publish --locked
```

### Tarball Contents

The package must not include local agent directories, editor state, build
outputs, private configs, broker session files, tokens, or machine-specific
paths. The `include` list in `Cargo.toml` is the source of truth — it
enumerates exactly what ships and fails closed for everything else.

Verify after every change that touches the repo root:

```bash
cargo package --allow-dirty --no-verify --list | head
```
