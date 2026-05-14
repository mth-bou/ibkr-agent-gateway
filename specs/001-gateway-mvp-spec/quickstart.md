# Quickstart: IBKR Agent Gateway Read-Only MVP

This quickstart describes the expected validation flow for the first
implementation increment. Commands may be adjusted during implementation, but
the outcomes must remain aligned with the contracts.

## 1. Prepare the Local Broker Session

1. Start the Interactive Brokers Client Portal Gateway locally.
2. Complete the manual broker authentication flow outside this gateway.
3. Confirm the broker gateway is reachable on the configured local URL.

## 2. Configure Read-Only Gateway Mode

Create a local config using [contracts/config.md](./contracts/config.md) as the
shape. Keep:

- `server.mode: local`
- `safety.write_tools_enabled: false`
- only read scopes under `auth.enabled_scopes`
- SQLite audit storage enabled

## 3. Run Static Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Expected result: formatting, linting, unit tests, integration tests, audit
redaction tests, and offline fixture tests pass.

## 4. Validate CLI Read-Only Behavior

```bash
ibkr-agent health --config ./config/local.yaml
ibkr-agent accounts list --config ./config/local.yaml
ibkr-agent positions list --account U1234567 --config ./config/local.yaml
ibkr-agent contracts search AAPL --asset-class stock --currency USD --config ./config/local.yaml
ibkr-agent market snapshot --contract-id 265598 --config ./config/local.yaml
ibkr-agent orders list --account U1234567 --config ./config/local.yaml
ibkr-agent audit tail --limit 20 --config ./config/local.yaml
```

Expected result: read-only commands return structured data or clear
user-actionable refusals.

## 5. Validate Write Refusal

Attempt any preview, submit, cancel, or write-like command if it exists as a
placeholder.

Expected result: the gateway refuses the operation, returns a typed read-only
error, and writes an audit event.

## 6. Validate Local MCP

Start local MCP serving:

```bash
ibkr-agent mcp serve --transport stdio --config ./config/local.yaml
```

Use a local MCP-compatible client or inspector to verify:

- only read-only tools are listed
- every listed tool has a minimal read scope
- forbidden order preview/submit/cancel tools are absent
- missing-scope calls are denied and audited
- returned broker data contains no secrets

## 7. Validate Audit Review

Run allowed and denied operations, then inspect the audit tail.

Expected result: audit events show request correlation, tool name, scope,
decision, status, account hash when applicable, and redaction metadata.
