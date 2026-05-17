# Production Readiness

This checklist is for operators and developers preparing a real deployment of
`ibkr-agent-gateway`.

The package is intentionally conservative, but it is still unofficial software
for financial workflows. Treat production enablement as an explicit deployment
decision, not as a default mode.

## Current Deployment Status

Ready for production-like validation:

- SDK facade with fake and Client Portal Gateway backend constructors;
- read-only broker data paths;
- redacted audit storage and export;
- local MCP stdio tool registry;
- remote MCP OAuth/OIDC validation primitives;
- preview, paper, sidecar, provider compatibility, and live-gate domain logic.

Not sufficient on its own for unattended live trading:

- the CLI runner currently defaults to fake fixtures for local commands;
- live CLI commands record local gated lifecycle candidates and must not be
  treated as broker-side execution;
- real broker write adapters must return broker-generated order ids before live
  submit/cancel wording or automation is promoted.

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
- approved preview/approval records;
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

## Package Publication

Before publishing or cutting a production artifact:

```bash
cargo package --allow-dirty --no-verify --list
cargo publish --dry-run --locked
```

The package must not include local agent directories, editor state, build
outputs, private configs, broker session files, tokens, or machine-specific
paths.
