# Quickstart: Validating MCP Tool Maturity Work

This quickstart is for implementation validation after each phase.

## Baseline Inventory

Run registry and schema tests:

```bash
cargo test --workspace --features unstable-internal-test-support --test contract_mcp_broker_tool_list
cargo test --workspace --features unstable-internal-test-support --test contract_mcp_schemas
```

Expected:

- pre-009 20-tool baseline remains documented and the implemented registry
  exposes the expanded 45-tool surface;
- generic write names remain forbidden;
- new tools appear only when their scopes are present.

## Phase 1 Read Tools

Use fake backend fixtures and MCP calls to validate:

```bash
cargo test --workspace --features unstable-internal-test-support --test integration_mcp_scope_denials
cargo test --workspace --features unstable-internal-test-support --test integration_mcp_redaction
cargo test --workspace --features unstable-internal-test-support --test integration_audit_scope_denials
cargo test --workspace --features unstable-internal-test-support --test replay_audit_redaction
```

Expected:

- `ibkr_pnl_daily`, `ibkr_pnl_realtime`, `ibkr_orders_history`,
  `ibkr_account_metadata`, `ibkr_kill_switch_status`, `ibkr_limits_status`,
  `ibkr_audit_export`, and `ibkr_session_renew` are scope-gated;
- audit contains no raw account ids or secrets;
- export output passes existing secret-like material checks.

## Phase 2 Write Tools

Validate idempotency, pending write recovery, and gates:

```bash
cargo test --workspace --features unstable-internal-test-support --test integration_live_idempotency
cargo test --workspace --features unstable-internal-test-support --test integration_live_kill_switch
cargo test --workspace --features unstable-internal-test-support --test integration_mcp_live_submit
```

Add equivalent modify-specific tests before declaring Phase 2 complete.

Expected:

- paper modify works with explicit paper scope;
- live modify refuses when config, scope, approval, limits, or kill switch gates
  are missing;
- generic `ibkr_order_modify` remains forbidden.

## Full Gate

Run before finishing each implementation phase:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace --features unstable-internal-test-support
```
