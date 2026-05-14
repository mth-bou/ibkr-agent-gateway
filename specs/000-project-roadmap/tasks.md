# Tasks: IBKR Agent Gateway Complete Roadmap

**Input**: Design documents from `/specs/000-project-roadmap/`

**Prerequisites**: spec.md, plan.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Every feature spec must include unit tests, integration tests, contract tests, audit/redaction tests, and explicit forbidden-capability tests.

**Organization**: Roadmap tasks are grouped by future feature spec. Detailed task lists live in each numbered feature spec.

## Format: `[ID] [P?] [Spec] Description`

- **[P]**: Can run in parallel with other roadmap tasks.
- **[Spec]**: Target feature spec.
- Every task includes at least one exact file or folder path.

## Phase 0: Product Roadmap Baseline

- [X] R001 Create full roadmap documentation in `specs/000-project-roadmap/spec.md`
- [X] R002 Create architecture and crate boundary plan in `specs/000-project-roadmap/plan.md`
- [X] R003 Create complete roadmap data model in `specs/000-project-roadmap/data-model.md`
- [X] R004 [P] Create architecture boundary contract in `specs/000-project-roadmap/contracts/architecture-boundaries.md`
- [X] R005 [P] Create feature roadmap contract in `specs/000-project-roadmap/contracts/feature-roadmap.md`
- [X] R006 [P] Create scope contract in `specs/000-project-roadmap/contracts/scopes.md`
- [X] R007 [P] Create order lifecycle contract in `specs/000-project-roadmap/contracts/order-lifecycle.md`
- [X] R008 [P] Create sidecar relay contract in `specs/000-project-roadmap/contracts/sidecar-relay.md`
- [X] R009 [P] Create provider compatibility contract in `specs/000-project-roadmap/contracts/provider-compatibility.md`

## Phase 1: Spec 001 - Local Read-Only MVP

- [X] R010 [001] Implement `specs/001-gateway-mvp-spec/tasks.md` completely before any write-capable spec begins
- [X] R011 [001] Validate no order preview, submit, cancel, sidecar, remote MCP, or live trading code path exists in `crates/`
- [X] R012 [001] Validate local MCP/CLI/audit/fake-backend acceptance through `cargo test --workspace`

## Phase 2: Spec 002 - Order Preview and Risk

- [X] R013 [002] Create `specs/002-order-preview-risk/spec.md` with preview-only scope and forbidden submit/cancel/live behavior
- [X] R014 [002] Add `crates/ibkr-risk/` for deterministic policies and `crates/ibkr-orders/` for preview models
- [X] R015 [002] Add `OrderIntent`, `RiskPolicy`, `RiskCheckResult`, `ValidatedOrder`, and `OrderPreview` contracts in `specs/002-order-preview-risk/contracts/`
- [X] R016 [002] Add MCP/CLI preview tools that cannot submit in `crates/ibkr-mcp/src/tools/orders.rs` and `crates/ibkr-cli/src/commands/orders.rs`
- [X] R017 [002] Add forbidden submit/cancel tests in `tests/contract_order_preview_no_submit.rs`

## Phase 3: Spec 003 - Paper Submit and Approval

- [X] R018 [003] Create `specs/003-paper-submit-approval/spec.md` with paper-only write scope
- [X] R019 [003] Add `crates/ibkr-approval/` for explicit approval records
- [X] R020 [003] Implement paper submit/cancel idempotency in `crates/ibkr-orders/src/paper_submit.rs`, `crates/ibkr-orders/src/paper_cancel.rs`, and `crates/ibkr-orders/src/idempotency.rs`
- [X] R021 [003] Add order lifecycle state machine tests in `tests/integration_order_lifecycle_paper.rs`
- [X] R022 [003] Add live trading forbidden tests in `tests/contract_paper_no_live.rs`

## Phase 4: Spec 004 - Remote MCP OAuth/OIDC

- [ ] R023 [004] Create `specs/004-remote-mcp-oauth/spec.md` with HTTP MCP and OAuth/OIDC boundaries
- [ ] R024 [004] Add `crates/ibkr-oauth/` for issuer, audience, signature, expiry, and scope validation
- [ ] R025 [004] Add HTTP MCP transport in `crates/ibkr-mcp/src/http_server.rs`
- [ ] R026 [004] Add remote auth denial tests in `tests/contract_remote_oauth_denials.rs`
- [ ] R027 [004] Add auth metadata documentation in `docs/remote-mcp-oauth.md`

## Phase 5: Spec 005 - Sidecar Relay

- [ ] R028 [005] Create `specs/005-sidecar-relay/spec.md` with local sidecar and remote relay scope
- [ ] R029 [005] Add `crates/ibkr-sidecar/` for local sidecar client and remote relay protocol
- [ ] R030 [005] Add sidecar heartbeat/disconnect tests in `tests/integration_sidecar_relay.rs`
- [ ] R031 [005] Add sidecar secret-leak tests in `tests/replay_sidecar_secret_scan.rs`
- [ ] R032 [005] Add sidecar runbook in `docs/sidecar-relay.md`

## Phase 6: Spec 006 - Provider Compatibility

- [ ] R033 [006] Create `specs/006-provider-compatibility/spec.md`
- [ ] R034 [006] Add `crates/ibkr-provider-compat/` for MCP compatibility fixtures and snapshots
- [ ] R035 [006] Add OpenAI remote MCP compatibility snapshots in `tests/provider_openai_mcp.rs`
- [ ] R036 [006] Add Anthropic remote MCP compatibility snapshots in `tests/provider_anthropic_mcp.rs`
- [ ] R037 [006] Add Cursor/Continue/local inspector compatibility snapshots in `tests/provider_mcp_clients.rs`

## Phase 7: Spec 007 - Live Trading Gated

- [ ] R038 [007] Create `specs/007-live-trading-gated/spec.md` with live-gated behavior only
- [ ] R039 [007] Add live account allowlist validation in `crates/ibkr-config/src/live.rs`
- [ ] R040 [007] Add live risk gates in `crates/ibkr-risk/src/live.rs`
- [ ] R041 [007] Add kill switch in `crates/ibkr-orders/src/kill_switch.rs`
- [ ] R042 [007] Add audit availability gate for live writes in `crates/ibkr-orders/src/submit.rs`
- [ ] R043 [007] Add live-disabled-by-default tests in `tests/contract_live_disabled_by_default.rs`

## Phase 8: Spec 008 - Operations Hardening

- [ ] R044 [008] Create `specs/008-operations-hardening/spec.md` for packaging, observability, audit export, replay, and runbooks
- [ ] R045 [008] Add structured metrics in `crates/ibkr-audit/src/metrics.rs` and `crates/ibkr-mcp/src/metrics.rs`
- [ ] R046 [008] Add audit JSONL export in `crates/ibkr-audit/src/export.rs`
- [ ] R047 [008] Add deployment docs in `docs/deployment.md`
- [ ] R048 [008] Add incident/runbook docs in `docs/runbooks.md`

## Dependencies & Execution Order

- `001` must complete before `002`.
- `002` must complete before `003`.
- `003` should complete before `007`.
- `004` must complete before `005` and `006` remote provider compatibility.
- `005` depends on `004` but not on `007`.
- `006` depends on the MCP transport and auth mode it tests.
- `007` depends on `002`, `003`, `004`, and audit maturity from `001`.
- `008` can begin partially after `001`, but production hardening for live trading depends on `007`.
