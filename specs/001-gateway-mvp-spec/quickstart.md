# Quickstart: IBKR Agent Gateway Read-Only MVP

This quickstart describes the expected validation flow for the first implementation increment. Commands may be adjusted during implementation, but the outcomes must remain aligned with the contracts.

## 1. Prepare the Local Broker Session

1. Start the Interactive Brokers Client Portal Gateway locally.
2. Complete the manual broker authentication flow outside this gateway.
3. Confirm the broker gateway is reachable on the configured local URL.
4. Keep the Client Portal Gateway URL local unless using the fake backend.

## 2. Configure Read-Only Gateway Mode

Create a local config using [contracts/config.md](./contracts/config.md) as the shape. Keep:

- `server.mode: local`
- `safety.write_tools_enabled: false`
- `safety.remote_public_mcp_enabled: false`
- `safety.sidecar_enabled: false`
- only read scopes under `auth.enabled_scopes`
- SQLite audit storage enabled
- HMAC account hashing enabled
- `broker.client_portal_gateway.verify_tls: false` only for localhost URLs
- market-data stale/delayed policy explicit

## 3. Run Static Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Expected result: formatting, linting, unit tests, integration tests, audit redaction tests, keepalive tests, secret scans, and offline fixture tests pass.

## 4. Validate CLI Read-Only Behavior

```bash
ibkr-agent health --config ./config/local.yaml
ibkr-agent backend status --config ./config/local.yaml
ibkr-agent session requirements --config ./config/local.yaml
ibkr-agent accounts list --config ./config/local.yaml
ibkr-agent account summary --account U1234567 --config ./config/local.yaml
ibkr-agent positions list --account U1234567 --config ./config/local.yaml
ibkr-agent portfolio snapshot --account U1234567 --config ./config/local.yaml
ibkr-agent contracts search AAPL --asset-class stock --currency USD --exchange SMART --config ./config/local.yaml
ibkr-agent contracts resolve AAPL --asset-class stock --currency USD --exchange SMART --config ./config/local.yaml
ibkr-agent market snapshot --contract-id 265598 --config ./config/local.yaml
ibkr-agent market bars --contract-id 265598 --duration "1 D" --bar-size "5 mins" --config ./config/local.yaml
ibkr-agent orders list --account U1234567 --config ./config/local.yaml
ibkr-agent orders status --account U1234567 --broker-order-id 123 --config ./config/local.yaml
ibkr-agent executions list --account U1234567 --from 2026-05-14T00:00:00Z --config ./config/local.yaml
```

Expected result: read-only commands return structured data or clear user-actionable refusals.

## 5. Validate Keepalive and Session Refusals

Run fake backend scenarios for:

- session usable;
- session missing;
- session expired;
- keepalive success;
- keepalive failure.

Expected result: broker-backed calls fail closed when session is missing/expired, and audit contains `backend.session.checked` plus `backend.session.changed` when state changes.

## 6. Validate Write Refusal

Attempt preview, submit, cancel, modify, approve, or write-like commands if placeholders exist.

Expected result: the gateway refuses the operation, returns a typed read-only error, and writes an audit event.

## 7. Validate Local MCP

Start local MCP serving:

```bash
ibkr-agent mcp serve --transport stdio --config ./config/local.yaml
```

Use a local MCP-compatible client or inspector to verify:

- only read-only broker tools are listed before US4;
- `ibkr_audit_tail` appears only after US4 is implemented;
- every listed tool has a minimal read scope;
- forbidden order preview/submit/cancel/modify/approve tools are absent;
- forbidden tool dispatch attempts are refused and audited;
- missing-scope calls are denied and audited;
- returned broker data contains no secrets.

## 8. Validate Audit Review

After US4, run allowed and denied operations, then inspect the audit tail.

```bash
ibkr-agent audit tail --limit 20 --config ./config/local.yaml
```

Expected result: audit events show request correlation, tool name, scope, decision, status, HMAC account hash when applicable, and redaction metadata.
