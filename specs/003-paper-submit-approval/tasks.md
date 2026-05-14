# Tasks: Paper Submit and Approval Workflow

## Phase 1: Approval and Idempotency

- [ ] T001 Add approval models in `crates/ibkr-approval/src/model.rs`.
- [ ] T002 Add idempotency models in `crates/ibkr-orders/src/idempotency.rs`.
- [ ] T003 Add paper account allowlist config in `crates/ibkr-config/src/paper.rs`.
- [ ] T004 Add paper scopes in `crates/ibkr-auth/src/scopes.rs`.
- [ ] T005 Add SQLite tables for approvals/idempotency in `crates/ibkr-audit/migrations/0002_paper_orders.sql`.

## Phase 2: Paper Submit/Cancel

- [ ] T010 Implement approval creation/read service in `crates/ibkr-approval/src/service.rs`.
- [ ] T011 Implement paper submit flow in `crates/ibkr-orders/src/paper_submit.rs`.
- [ ] T012 Implement paper cancel flow in `crates/ibkr-orders/src/paper_cancel.rs`.
- [ ] T013 Implement lifecycle polling/streaming in `crates/ibkr-orders/src/lifecycle.rs`.
- [ ] T014 Implement CPAPI paper submit/cancel adapter in `crates/ibkr-cpapi/src/orders_write.rs`.

## Phase 3: CLI, MCP, and Audit

- [ ] T020 Add CLI approval commands in `crates/ibkr-cli/src/commands/approvals.rs`.
- [ ] T021 Add CLI paper submit/cancel commands in `crates/ibkr-cli/src/commands/orders_paper.rs`.
- [ ] T022 Add MCP paper submit/cancel tools in `crates/ibkr-mcp/src/tools/orders_paper.rs`.
- [ ] T023 Add approval/submit/cancel/lifecycle audit events in `crates/ibkr-orders/src/audit.rs`.
- [ ] T024 Keep live submit/cancel absent/refused in `crates/ibkr-mcp/src/registry.rs`.

## Phase 4: Tests and Docs

- [ ] T030 Add approval-required tests in `tests/integration_paper_approval.rs`.
- [ ] T031 Add idempotency replay tests in `tests/replay_paper_idempotency.rs`.
- [ ] T032 Add paper lifecycle fixture tests in `tests/integration_order_lifecycle_paper.rs`.
- [ ] T033 Add live-forbidden tests in `tests/contract_paper_no_live.rs`.
- [ ] T034 Add audit tests in `tests/integration_paper_order_audit.rs`.
- [ ] T035 Document paper order workflow in `docs/paper-orders.md`.
