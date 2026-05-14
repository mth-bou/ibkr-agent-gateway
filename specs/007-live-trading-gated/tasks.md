# Tasks: Live Trading Gated Enablement

## Phase 1: Live Gates

- [ ] T001 Add live mode config disabled by default in `crates/ibkr-config/src/live.rs`.
- [ ] T002 Add live account allowlist validation in `crates/ibkr-config/src/live.rs`.
- [ ] T003 Add live scopes in `crates/ibkr-auth/src/scopes.rs`.
- [ ] T004 Add kill switch model and storage in `crates/ibkr-orders/src/kill_switch.rs`.
- [ ] T005 Add live gate model in `crates/ibkr-risk/src/live_gate.rs`.

## Phase 2: Limits and Policies

- [ ] T010 Add notional, quantity, symbol, asset-class, frequency, and session limit policies in `crates/ibkr-risk/src/live_limits.rs`.
- [ ] T011 Add missing-gate refusal matrix in `crates/ibkr-risk/src/live_refusals.rs`.
- [ ] T012 Add live audit retention config in `crates/ibkr-config/src/audit_retention.rs`.
- [ ] T013 Add paper-to-live migration checks in `crates/ibkr-orders/src/live_migration.rs`.

## Phase 3: Live Submit/Cancel

- [ ] T020 Add live submit behind all gates in `crates/ibkr-orders/src/live_submit.rs`.
- [ ] T021 Add live cancel behind all gates in `crates/ibkr-orders/src/live_cancel.rs`.
- [ ] T022 Add lifecycle tracking and execution correlation in `crates/ibkr-orders/src/lifecycle.rs`.
- [ ] T023 Add emergency disable behavior in `crates/ibkr-orders/src/kill_switch.rs`.
- [ ] T024 Add CLI/MCP live commands only when enabled in `crates/ibkr-cli/src/commands/orders_live.rs` and `crates/ibkr-mcp/src/tools/orders_live.rs`.

## Phase 4: Tests and Docs

- [ ] T030 Add tool discovery tests for disabled/enabled live states in `tests/contract_live_tool_discovery.rs`.
- [ ] T031 Add missing gate tests in `tests/integration_live_gate_refusals.rs`.
- [ ] T032 Add kill switch tests in `tests/integration_live_kill_switch.rs`.
- [ ] T033 Add live disabled by default tests in `tests/contract_live_disabled_by_default.rs`.
- [ ] T034 Add paper-to-live migration checklist in `docs/paper-to-live.md`.
- [ ] T035 Add operator runbook and incident review template in `docs/live-runbook.md`.
