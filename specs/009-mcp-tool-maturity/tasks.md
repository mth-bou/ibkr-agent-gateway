# Tasks: MCP Tool Maturity Expansion

**Input**: Design documents from `specs/009-mcp-tool-maturity/`

**Tests**: Required. Each tool needs registry, schema, scope-denial, handler,
fake-backend, CPAPI mapping or refusal, audit redaction, docs, and contract
coverage.

**Format**: `[ID] [P?] [Story?] Description`

## Phase 0: Baseline Registry and Contracts

**Purpose**: Capture the current MCP tool surface before adding tools.

- [X] T001 Update or add registry inventory assertions in `tests/contract_mcp_broker_tool_list.rs`
- [X] T002 Add schema expectations for the maturity contract in `tests/contract_mcp_schemas.rs`
- [X] T003 Document current and target tool mapping in `docs/mcp-local.md`
- [X] T004 Document new scope plan in `docs/scopes.md`
- [X] T005 Verify `FORBIDDEN_TOOL_NAMES` in `src/internal/mcp/registry.rs` still includes generic `ibkr_order_modify`

## Phase 1: Consultative and Safety Read Tools

**Purpose**: Add high-value read-only and operational visibility tools.

### Domain and Backend

- [X] T006 [P] [US1] Add PnL models in `src/internal/domain/pnl.rs` and export them from `src/internal/domain/mod.rs`
- [X] T007 [P] [US1] Add order history models in `src/internal/domain/order_history.rs` and export them
- [X] T008 [P] [US1] Add account capability profile models in `src/internal/domain/account.rs`
- [X] T009 [P] [US2] Add limit status DTOs in `src/internal/risk/live_limits.rs`
- [X] T010 [US1] Extend `IbkrBackend` in `src/internal/backend/trait.rs` with `pnl_daily`, `pnl_realtime`, `orders_history`, and `account_metadata`
- [X] T011 [US1] Extend `FakeBackend` in `src/internal/backend/fake.rs` for PnL, order history, and metadata fixtures
- [X] T012 [US1] Extend `ClientPortalBackend` in `src/internal/backend/client_portal.rs` for PnL, order history, and metadata calls
- [X] T013 [US1] Add CPAPI response models in `src/internal/cpapi/models.rs`
- [X] T014 [US1] Add CPAPI client methods in `src/internal/cpapi/client.rs`
- [X] T015 [US1] Add CPAPI mappers and safe refusals in `src/internal/cpapi/mapper.rs`

### Scopes and MCP Registry

- [X] T016 [US2] Add `AUDIT_EXPORT`, options for `RISK_READ`, and future read scopes in `src/internal/auth/scopes.rs`
- [X] T017 [US1] Add tool schema helpers in `src/internal/mcp/tools/portfolio.rs`, `src/internal/mcp/tools/accounts.rs`, and `src/internal/mcp/tools/orders.rs`
- [X] T018 [US2] Add safety/operations schemas in `src/internal/mcp/tools/health.rs` and `src/internal/mcp/tools/audit.rs`
- [X] T019 [US1] Register Phase 1 tools in `src/internal/mcp/registry.rs`
- [X] T020 [US1] Add handler branches in `src/cli/commands/mcp.rs`

### Audit and Tests

- [X] T021 [P] [US1] Add fixtures under `tests/fixtures/cpapi/` for PnL, order history, and account metadata
- [X] T022 [P] [US1] Add integration tests in `tests/integration_pnl_read.rs`
- [X] T023 [P] [US1] Add integration tests in `tests/integration_orders_history.rs`
- [X] T024 [P] [US1] Add integration tests in `tests/integration_account_metadata.rs`
- [X] T025 [P] [US2] Add integration tests in `tests/integration_mcp_safety_status.rs`
- [X] T026 [P] [US2] Add MCP audit export tests in `tests/integration_mcp_audit_export.rs`
- [X] T027 [US1] Extend audit redaction tests in `tests/replay_audit_redaction.rs`
- [X] T028 [US1] Update docs in `docs/tools.md`, `docs/mcp-local.md`, and `docs/scopes.md`
- [X] T029 [US1] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace`

## Phase 2: Modify Tools and Protective Order Types

**Purpose**: Add safe order adjustment and stop-style order support.

### Domain and Risk

- [X] T030 [US3] Extend `PreviewOrderType` and order fields in `src/internal/domain/order_preview.rs`
- [X] T031 [US3] Update order validation in `src/internal/risk/validate.rs`
- [X] T032 [US3] Update deterministic risk checks in `src/internal/risk/checks.rs`
- [X] T033 [US3] Update live limit price/reference checks in `src/internal/risk/live_limits.rs`
- [X] T034 [US3] Update order preview schema generation in `src/internal/mcp/schemas.rs`
- [X] T035 [US3] Update CLI order preview parsing in `src/cli/commands/orders_preview.rs`
- [X] T036 [US3] Update MCP preview parsing in `src/internal/mcp/order_workflows.rs`

### Writer Boundaries

- [X] T037 [US3] Add paper modify request/result and writer trait in `src/internal/orders/paper_modify.rs`
- [X] T038 [US3] Add live modify request/result and writer trait in `src/internal/orders/live_modify.rs`
- [X] T039 [US3] Export modify modules from `src/internal/orders/mod.rs`
- [X] T040 [US3] Add CPAPI modify body builders in `src/internal/cpapi/live_writer.rs`
- [X] T041 [US3] Add paper/local candidate modify writer behavior in `src/internal/orders/live_writer.rs` or a dedicated writer module
- [X] T042 [US3] Add pending idempotency recovery context variants in `src/internal/audit/sqlite.rs`

### Scopes, Registry, Handlers

- [X] T043 [US3] Add `ORDERS_PAPER_MODIFY` and `ORDERS_LIVE_MODIFY` in `src/internal/auth/scopes.rs`
- [X] T044 [US3] Add MCP schemas in `src/internal/mcp/tools/orders_paper.rs` and `src/internal/mcp/tools/orders_live.rs`
- [X] T045 [US3] Register modify tools in `src/internal/mcp/registry.rs`
- [X] T046 [US3] Add paper modify handler in `src/internal/mcp/order_workflows.rs`
- [X] T047 [US3] Add live modify handler in `src/internal/mcp/live_orders.rs`
- [X] T048 [US3] Add stdio dispatch branches in `src/cli/commands/mcp.rs`
- [X] T049 [US3] Keep generic `ibkr_order_modify` forbidden in tests and docs

### Tests and Docs

- [X] T050 [P] [US3] Add order type validation tests in `tests/integration_order_preview_advanced_types.rs`
- [X] T051 [P] [US3] Add CPAPI writer contract tests in `tests/contract_cpapi_live_writer.rs`
- [X] T052 [P] [US3] Add paper modify lifecycle tests in `tests/integration_order_modify_paper.rs`
- [X] T053 [P] [US3] Add live modify gate tests in `tests/integration_order_modify_live.rs`
- [X] T054 [P] [US3] Add idempotency/recovery tests in `tests/integration_order_modify_idempotency.rs`
- [X] T055 [US3] Update `docs/order-preview.md`, `docs/paper-orders.md`, and `docs/live-runbook.md`
- [X] T056 [US3] Run full verification gates

## Phase 3: Advanced Market Research

**Purpose**: Add read-only options, greeks, depth, and scanner tools.

- [ ] T057 [P] [US4] Add options models in `src/internal/domain/options.rs`
- [ ] T058 [P] [US4] Add market depth models in `src/internal/domain/market_depth.rs`
- [ ] T059 [P] [US4] Add scanner models in `src/internal/domain/scanner.rs`
- [ ] T060 [US4] Extend `IbkrBackend` with options chain, greeks, market depth, and scanner methods
- [ ] T061 [US4] Add fake backend fixtures and methods in `src/internal/backend/fake.rs`
- [ ] T062 [US4] Add CPAPI client/mapper/model support in `src/internal/cpapi/`
- [ ] T063 [US4] Add scopes in `src/internal/auth/scopes.rs`
- [ ] T064 [US4] Add MCP schemas in `src/internal/mcp/tools/market.rs`
- [ ] T065 [US4] Register tools in `src/internal/mcp/registry.rs`
- [ ] T066 [US4] Add handler branches in `src/cli/commands/mcp.rs`
- [ ] T067 [P] [US4] Add tests in `tests/integration_options_read.rs`
- [ ] T068 [P] [US4] Add tests in `tests/integration_market_depth.rs`
- [ ] T069 [P] [US4] Add tests in `tests/integration_scanner_run.rs`
- [ ] T070 [US4] Update docs in `docs/tools.md` and `docs/mcp-local.md`
- [ ] T071 [US4] Run full verification gates

## Phase 4: Bracket and OCA Groups

**Purpose**: Add coordinated group order workflows after single-order protection is stable.

- [ ] T072 [US5] Add group order domain models in `src/internal/domain/order_group.rs`
- [ ] T073 [US5] Add group preview service in `src/internal/orders/group_preview.rs`
- [ ] T074 [US5] Add group lifecycle storage migration in `src/internal/audit/migrations/`
- [ ] T075 [US5] Add group writer traits in `src/internal/orders/group_writer.rs`
- [ ] T076 [US5] Add paper group submit in `src/internal/orders/group_paper_submit.rs`
- [ ] T077 [US5] Add live group submit in `src/internal/orders/group_live_submit.rs`
- [ ] T078 [US5] Add MCP group schemas in `src/internal/mcp/tools/order_groups.rs`
- [ ] T079 [US5] Register group tools in `src/internal/mcp/registry.rs`
- [ ] T080 [US5] Add MCP handlers in `src/internal/mcp/order_groups.rs`
- [ ] T081 [P] [US5] Add bracket preview tests in `tests/integration_bracket_preview.rs`
- [ ] T082 [P] [US5] Add paper bracket submit tests in `tests/integration_bracket_paper.rs`
- [ ] T083 [P] [US5] Add live bracket gate tests in `tests/integration_bracket_live.rs`
- [ ] T084 [US5] Document bracket/OCA workflows in `docs/bracket-orders.md`
- [ ] T085 [US5] Run full verification gates

## Phase 5: Contextual Data and MCP Approval Creation

**Purpose**: Add lower-priority context tools and MCP-native approval records.

- [ ] T086 [P] [US6] Add news models in `src/internal/domain/news.rs`
- [ ] T087 [P] [US6] Add fundamentals models in `src/internal/domain/fundamentals.rs`
- [ ] T088 [P] [US6] Add calendar/session models in `src/internal/domain/calendar.rs`
- [ ] T089 [P] [US6] Add currency and transfer models in `src/internal/domain/account_activity.rs`
- [ ] T090 [US6] Extend `IbkrBackend` for news, fundamentals, sessions, currency, and transfers
- [ ] T091 [US6] Add fake backend fixtures and CPAPI mapping for contextual reads
- [ ] T092 [US6] Add dedicated read scopes in `src/internal/auth/scopes.rs`
- [ ] T093 [US6] Add MCP schemas and registry entries for contextual tools
- [ ] T094 [US6] Add `ibkr_approvals_create` schema and scope in `src/internal/mcp/tools/approvals.rs`
- [ ] T095 [US6] Add MCP approval creation handler using `ApprovalService` and `SqliteAuditWriter`
- [ ] T096 [P] [US6] Add tests in `tests/integration_news_fundamentals.rs`
- [ ] T097 [P] [US6] Add tests in `tests/integration_calendar_currency.rs`
- [ ] T098 [P] [US6] Add tests in `tests/integration_transfer_history.rs`
- [ ] T099 [P] [US6] Add tests in `tests/integration_mcp_approval_create.rs`
- [ ] T100 [US6] Update docs in `docs/provider-approval-ux.md`, `docs/tools.md`, and `docs/scopes.md`
- [ ] T101 [US6] Run full verification gates

## Final Readiness

- [ ] T102 Review all new public exports in `src/public/*.rs`
- [ ] T103 Update `docs/public-api.md`
- [ ] T104 Update `docs/production-readiness.md`
- [ ] T105 Run `cargo package --allow-dirty --no-verify --list` and inspect for accidental fixture/secret/package residue
- [ ] T106 Run final `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace`
