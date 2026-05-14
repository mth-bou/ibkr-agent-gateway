# Tasks: Order Preview and Deterministic Risk Engine

## Phase 1: Domain, Config, and Risk Models

- [X] T001 Add `crates/ibkr-risk/Cargo.toml` and `crates/ibkr-risk/src/lib.rs`.
- [X] T002 Add `crates/ibkr-orders/Cargo.toml` and `crates/ibkr-orders/src/lib.rs`.
- [X] T003 Add `OrderIntent`, `ValidatedOrder`, and `OrderPreview` in `crates/ibkr-domain/src/order_preview.rs`.
- [X] T004 Add `RiskPolicy`, `RiskWarning`, and `RiskRefusal` in `crates/ibkr-risk/src/policy.rs`.
- [X] T005 Add preview-disabled-by-default config in `crates/ibkr-config/src/order_preview.rs`.
- [X] T006 Add preview scopes to `crates/ibkr-auth/src/scopes.rs`.

## Phase 2: Validation and Preview

- [X] T010 Implement deterministic order-intent validation in `crates/ibkr-risk/src/validate.rs`.
- [X] T011 Implement notional, asset-class, account-mode, side, quantity, order-type, and price checks in `crates/ibkr-risk/src/checks.rs`.
- [X] T012 Implement validated order construction in `crates/ibkr-orders/src/validated_order.rs`.
- [X] T013 Implement non-executable preview service in `crates/ibkr-orders/src/preview.rs`.
- [X] T014 Add CPAPI preview/read-only estimate mapping if supported in `crates/ibkr-cpapi/src/preview.rs`.

## Phase 3: CLI, MCP, and Audit

- [ ] T020 Add CLI `orders preview` command in `crates/ibkr-cli/src/commands/orders_preview.rs`.
- [ ] T021 Add MCP `ibkr_order_preview` tool in `crates/ibkr-mcp/src/tools/order_preview.rs`.
- [ ] T022 Add preview schema generation in `crates/ibkr-mcp/src/schemas.rs`.
- [ ] T023 Add audit events for intent validation, risk refusals, and preview creation in `crates/ibkr-orders/src/audit.rs`.
- [ ] T024 Ensure submit/cancel/approve remain absent/refused in `crates/ibkr-mcp/src/registry.rs` and `crates/ibkr-cli/src/commands/orders.rs`.

## Phase 4: Tests and Docs

- [ ] T030 Add risk unit tests in `crates/ibkr-risk/src/policy.rs`.
- [ ] T031 Add preview fixture tests in `tests/integration_order_preview.rs`.
- [ ] T032 Add forbidden submit/cancel tests in `tests/contract_order_preview_no_submit.rs`.
- [ ] T033 Add MCP/CLI schema snapshots in `tests/contract_order_preview_schemas.rs`.
- [ ] T034 Add audit tests for preview success/refusal in `tests/integration_order_preview_audit.rs`.
- [ ] T035 Document preview-only workflow in `docs/order-preview.md`.
