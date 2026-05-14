# Tasks: IBKR Agent Gateway Read-Only MVP

**Input**: Design documents from `/specs/001-gateway-mvp-spec/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md, and `/specs/000-project-roadmap/`

**Tests**: Required for this feature. The constitution and spec require unit, contract, integration, MCP schema/tool-list, audit redaction, keepalive, scope denial, secret scan, and fixture replay coverage.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel because it touches different files and does not depend on an incomplete task.
- **[Story]**: User story label for story phases only.
- Every task includes at least one exact file path.
- Rust integration tests use top-level Cargo-discoverable files under `tests/` unless an explicit harness is created.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the Rust workspace, crate boundaries, toolchain pin, Cargo-discoverable tests, and baseline documentation required by every story.

- [ ] T001 Create Rust workspace manifest with member crates in `Cargo.toml`
- [ ] T002 Pin selected stable Rust toolchain and edition defaults in `rust-toolchain.toml` and `Cargo.toml`
- [ ] T003 [P] Add shared Cargo lint and test aliases in `.cargo/config.toml`
- [ ] T004 [P] Create crate manifests for `crates/ibkr-domain/Cargo.toml`, `crates/ibkr-backend/Cargo.toml`, `crates/ibkr-cpapi/Cargo.toml`, `crates/ibkr-config/Cargo.toml`, `crates/ibkr-auth/Cargo.toml`, `crates/ibkr-audit/Cargo.toml`, `crates/ibkr-mcp/Cargo.toml`, and `crates/ibkr-cli/Cargo.toml`
- [ ] T005 [P] Create initial crate entrypoints in `crates/ibkr-domain/src/lib.rs`, `crates/ibkr-backend/src/lib.rs`, `crates/ibkr-cpapi/src/lib.rs`, `crates/ibkr-config/src/lib.rs`, `crates/ibkr-auth/src/lib.rs`, `crates/ibkr-audit/src/lib.rs`, `crates/ibkr-mcp/src/lib.rs`, and `crates/ibkr-cli/src/main.rs`
- [ ] T006 [P] Create repository fixture directory in `tests/fixtures/cpapi/` and top-level Cargo-discoverable test placeholders in `tests/contract_cli_us1.rs`, `tests/contract_cli_us2.rs`, `tests/contract_mcp_broker_tool_list.rs`, `tests/contract_mcp_schemas.rs`, `tests/contract_audit_events.rs`, `tests/integration_backend_status.rs`, and `tests/replay_secret_scan.rs`
- [ ] T007 [P] Add local config example matching the config contract in `config/local.example.yaml`
- [ ] T008 [P] Add getting-started documentation scaffold in `docs/getting-started-local.md`
- [ ] T009 [P] Add broker gateway documentation scaffold in `docs/ibkr-client-portal-gateway.md`
- [ ] T010 [P] Add roadmap link and non-goals summary in `README.md`
- [ ] T011 Verify workspace bootstraps with empty crates by running checks documented in `specs/001-gateway-mvp-spec/quickstart.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement shared domain, errors, configuration, scopes, backend traits, market-data policy, and audit recorder primitives that all user stories depend on.

**Critical**: No user story work can begin until this phase is complete.

- [ ] T012 [P] Implement typed identifiers and request correlation types in `crates/ibkr-domain/src/identifiers.rs`
- [ ] T013 [P] Implement decimal-safe money and quantity types in `crates/ibkr-domain/src/money.rs`
- [ ] T014 [P] Implement shared typed error codes from `contracts/error-codes.md` and user actions in `crates/ibkr-domain/src/error.rs`
- [ ] T015 [P] Implement broker account and session status models in `crates/ibkr-domain/src/account.rs`
- [ ] T016 [P] Implement contract models with MVP asset-class validation in `crates/ibkr-domain/src/contract.rs`
- [ ] T017 [P] Implement market snapshot, historical bars, and market-data status models in `crates/ibkr-domain/src/market.rs`
- [ ] T018 [P] Implement read-only order record models and forbid write states in `crates/ibkr-domain/src/order.rs`
- [ ] T019 Wire all domain modules and serde/schemars exports in `crates/ibkr-domain/src/lib.rs`
- [ ] T020 [P] Implement gateway configuration models and read-only validation outside the domain crate in `crates/ibkr-config/src/lib.rs`
- [ ] T021 [P] Implement TLS-bypass localhost-only validation in `crates/ibkr-config/src/validation.rs`
- [ ] T022 [P] Implement market-data policy config validation in `crates/ibkr-config/src/market_data.rs`
- [ ] T023 [P] Implement local scope constants and scope-set validation in `crates/ibkr-auth/src/scopes.rs`
- [ ] T024 [P] Implement local user identity, `AuthContextSource::LocalConfig`, and scope-check result types in `crates/ibkr-auth/src/local_user.rs`
- [ ] T025 Implement auth crate exports and deny-by-default helpers in `crates/ibkr-auth/src/lib.rs`
- [ ] T026 [P] Implement `IbkrBackend` read-only trait including keepalive/status/read methods in `crates/ibkr-backend/src/trait.rs`
- [ ] T027 [P] Implement fake backend fixture loader in `crates/ibkr-backend/src/fake.rs`
- [ ] T028 Wire backend crate exports in `crates/ibkr-backend/src/lib.rs`
- [ ] T029 [P] Implement audit event model and required event types in `crates/ibkr-audit/src/event.rs`
- [ ] T030 [P] Implement audit redaction and HMAC-SHA256 helpers in `crates/ibkr-audit/src/redaction.rs`
- [ ] T031 [P] Implement SQLite audit writer and migrations in `crates/ibkr-audit/src/sqlite.rs` and `crates/ibkr-audit/migrations/0001_audit_events.sql`
- [ ] T032 Implement shared record-only `AuditRecorder` service in `crates/ibkr-audit/src/recorder.rs`
- [ ] T033 Wire audit crate exports in `crates/ibkr-audit/src/lib.rs`
- [ ] T034 [P] Add unit tests for domain validation in `crates/ibkr-domain/src/lib.rs` and config validation in `crates/ibkr-config/src/lib.rs`
- [ ] T035 [P] Add unit tests for scope validation in `crates/ibkr-auth/src/lib.rs`
- [ ] T036 [P] Add unit tests for audit redaction and HMAC hashing in `crates/ibkr-audit/src/lib.rs`
- [ ] T037 [P] Add audit SQLite persistence unit/integration coverage in `tests/integration_audit_sqlite.rs`
- [ ] T038 Run foundational checks with `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace` from `Cargo.toml`

**Checkpoint**: Domain, scopes, backend trait, config, market-data policy, and shared audit recorder are ready for story implementation.

---

## Phase 3: User Story 1 - Verify Broker Session and Accounts (Priority: P1)

**Goal**: A local user can verify broker session usability, keepalive behavior, and list accessible accounts without leaking secrets.

**Independent Test**: With fake connected, missing-session, expired-session, keepalive-success, and keepalive-failure fixtures, status and account discovery return either safe account metadata or a manual-action required refusal.

### Tests for User Story 1

- [ ] T039 [P] [US1] Add fake connected account fixture in `tests/fixtures/cpapi/accounts_success.json`
- [ ] T040 [P] [US1] Add fake missing-session fixture in `tests/fixtures/cpapi/session_required.json`
- [ ] T041 [P] [US1] Add fake expired-session fixture in `tests/fixtures/cpapi/session_expired.json`
- [ ] T042 [P] [US1] Add fake keepalive fixtures in `tests/fixtures/cpapi/tickle_success.json` and `tests/fixtures/cpapi/tickle_session_expired.json`
- [ ] T043 [P] [US1] Add backend status integration tests in `tests/integration_backend_status.rs`
- [ ] T044 [P] [US1] Add account list integration tests in `tests/integration_accounts_list.rs`
- [ ] T045 [P] [US1] Add keepalive/session transition integration tests in `tests/integration_keepalive.rs`
- [ ] T046 [P] [US1] Add CLI contract tests for health, backend status, session requirements, and accounts list in `tests/contract_cli_us1.rs`
- [ ] T047 [P] [US1] Add audit/security tests for session and account errors in `tests/integration_audit_us1.rs`

### Implementation for User Story 1

- [ ] T048 [P] [US1] Implement Client Portal response models for session, keepalive, and accounts in `crates/ibkr-cpapi/src/models.rs`
- [ ] T049 [US1] Implement Client Portal HTTP client health, session, tickle/keepalive, and accounts calls in `crates/ibkr-cpapi/src/client.rs`
- [ ] T050 [US1] Implement CPAPI session/account/keepalive error mapping in `crates/ibkr-cpapi/src/mapper.rs`
- [ ] T051 [US1] Wire CPAPI crate exports in `crates/ibkr-cpapi/src/lib.rs`
- [ ] T052 [US1] Implement `ClientPortalBackend` health, session status, keepalive, and list accounts in `crates/ibkr-backend/src/client_portal.rs`
- [ ] T053 [US1] Add backend factory for client portal and fake backend in `crates/ibkr-backend/src/factory.rs`
- [ ] T054 [US1] Implement CLI health command in `crates/ibkr-cli/src/commands/health.rs`
- [ ] T055 [US1] Implement CLI backend status and session requirements commands in `crates/ibkr-cli/src/commands/backend.rs`
- [ ] T056 [US1] Implement CLI accounts list command in `crates/ibkr-cli/src/commands/accounts.rs`
- [ ] T057 [US1] Implement shared CLI JSON and human output formatting in `crates/ibkr-cli/src/output.rs`
- [ ] T058 [US1] Wire CLI command routing for US1 commands in `crates/ibkr-cli/src/main.rs`
- [ ] T059 [US1] Emit audit events for health, backend status, session requirements, keepalive, and accounts list using shared recorder in `crates/ibkr-cli/src/audit.rs`
- [ ] T060 [US1] Document US1 commands and expected missing-session/expired-session behavior in `docs/getting-started-local.md`

**Checkpoint**: US1 can be validated independently with CLI commands and fake fixtures before portfolio, market data, MCP, or audit tail work exists.

---

## Phase 4: User Story 2 - Inspect Portfolio and Market Data Read-Only (Priority: P2)

**Goal**: A local user can inspect account summary, positions, portfolio snapshot, supported stock/ETF contract candidates, market data, read-only orders, and executions without any write capability.

**Independent Test**: Offline fixtures return structured read-only data or clear refusals for missing account, ambiguous contract, unsupported asset class, stale market data, delayed data, historical bars unavailability, and write-like requests.

### Tests for User Story 2

- [ ] T061 [P] [US2] Add portfolio and positions fixtures in `tests/fixtures/cpapi/portfolio_snapshot.json` and `tests/fixtures/cpapi/positions_list.json`
- [ ] T062 [P] [US2] Add contract and market data fixtures in `tests/fixtures/cpapi/contracts_search_stock_etf.json`, `tests/fixtures/cpapi/contracts_ambiguous.json`, `tests/fixtures/cpapi/contracts_unsupported_asset_class.json`, `tests/fixtures/cpapi/market_snapshot_live.json`, `tests/fixtures/cpapi/market_snapshot_delayed.json`, `tests/fixtures/cpapi/market_snapshot_stale.json`, and `tests/fixtures/cpapi/historical_bars.json`
- [ ] T063 [P] [US2] Add orders and executions fixtures in `tests/fixtures/cpapi/orders_list.json`, `tests/fixtures/cpapi/order_status.json`, and `tests/fixtures/cpapi/executions_list.json`
- [ ] T064 [P] [US2] Add account context refusal tests in `tests/integration_account_context_refusals.rs`
- [ ] T065 [P] [US2] Add portfolio and positions integration tests in `tests/integration_portfolio_positions.rs`
- [ ] T066 [P] [US2] Add contract ambiguity, unsupported asset class, market snapshot, delayed/stale policy, and historical bars availability/refusal tests in `tests/integration_contracts_market.rs`
- [ ] T067 [P] [US2] Add read-only orders integration tests in `tests/integration_orders_readonly.rs`
- [ ] T068 [P] [US2] Add forbidden write refusal tests in `tests/integration_write_refusals.rs`
- [ ] T069 [P] [US2] Add CLI contract tests for portfolio, market, and orders commands in `tests/contract_cli_us2.rs`

### Implementation for User Story 2

- [ ] T070 [P] [US2] Implement CPAPI portfolio, positions, and summary response models in `crates/ibkr-cpapi/src/models.rs`
- [ ] T071 [US2] Implement CPAPI contract, market data, historical bars, orders, and executions response models in `crates/ibkr-cpapi/src/models.rs`
- [ ] T072 [US2] Implement CPAPI read calls for account summary, positions, portfolio, contracts, market data, historical bars, orders, and executions in `crates/ibkr-cpapi/src/client.rs`
- [ ] T073 [US2] Implement CPAPI read data mapping, asset-class filtering, market-data status, and ambiguity detection in `crates/ibkr-cpapi/src/mapper.rs`
- [ ] T074 [US2] Implement backend trait methods for account summary, positions, portfolio snapshot, contract search, contract resolve, market snapshot, historical bars, orders list, order status, and executions list in `crates/ibkr-backend/src/client_portal.rs`
- [ ] T075 [US2] Extend fake backend for all US2 read-only flows in `crates/ibkr-backend/src/fake.rs`
- [ ] T076 [US2] Implement account context validation helper in `crates/ibkr-backend/src/account_context.rs`
- [ ] T077 [US2] Implement market-data stale/delayed policy application in `crates/ibkr-backend/src/market_data_policy.rs`
- [ ] T078 [US2] Implement CLI account summary and portfolio commands in `crates/ibkr-cli/src/commands/account.rs` and `crates/ibkr-cli/src/commands/portfolio.rs`
- [ ] T079 [US2] Implement CLI positions command in `crates/ibkr-cli/src/commands/positions.rs`
- [ ] T080 [US2] Implement CLI contract commands in `crates/ibkr-cli/src/commands/contracts.rs`
- [ ] T081 [US2] Implement CLI market data commands in `crates/ibkr-cli/src/commands/market.rs`
- [ ] T082 [US2] Implement CLI read-only orders and executions commands in `crates/ibkr-cli/src/commands/orders.rs`
- [ ] T083 [US2] Add explicit forbidden write command refusals in `crates/ibkr-cli/src/commands/orders.rs`
- [ ] T084 [US2] Wire US2 CLI commands in `crates/ibkr-cli/src/main.rs`
- [ ] T085 [US2] Emit audit events for portfolio, market, order-read, ambiguity, unsupported asset class, stale-data, and write-refusal paths using shared recorder in `crates/ibkr-cli/src/audit.rs`
- [ ] T086 [US2] Document read-only data commands, supported asset classes, market-data freshness, and write refusal behavior in `docs/tools.md`

**Checkpoint**: US2 can be validated independently through CLI plus fake fixtures and must still expose no write path.

---

## Phase 5: User Story 3 - Use Read-Only MCP Tools from Local Agents (Priority: P3)

**Goal**: A local MCP-compatible client can list and call only read-only broker tools with strict schemas, minimal local scopes, safe outputs, and optional keepalive loop.

**Independent Test**: A local MCP client or tool-list test sees only read-only broker tools; missing scopes are denied; forbidden order preview/submit/cancel/modify/approve tools are absent or refused; audit tail is not exposed until US4.

### Tests for User Story 3

- [ ] T087 [P] [US3] Add MCP broker tool list snapshot test in `tests/contract_mcp_broker_tool_list.rs`
- [ ] T088 [P] [US3] Add MCP schema snapshot tests for all read-only broker tools in `tests/contract_mcp_schemas.rs`
- [ ] T089 [P] [US3] Add MCP missing-scope denial tests in `tests/integration_mcp_scope_denials.rs`
- [ ] T090 [P] [US3] Add MCP forbidden write tool absence/refusal tests in `tests/integration_mcp_write_refusals.rs`
- [ ] T091 [P] [US3] Add MCP safe-output redaction tests in `tests/integration_mcp_redaction.rs`
- [ ] T092 [P] [US3] Add MCP keepalive loop/session-expiry tests in `tests/integration_mcp_keepalive.rs`

### Implementation for User Story 3

- [ ] T093 [P] [US3] Implement MCP request and response schemas from domain models in `crates/ibkr-mcp/src/schemas.rs`
- [ ] T094 [US3] Implement MCP broker tool registry with only read-only broker tool names in `crates/ibkr-mcp/src/registry.rs`
- [ ] T095 [US3] Implement MCP local scope enforcement middleware in `crates/ibkr-mcp/src/scope_guard.rs`
- [ ] T096 [US3] Implement health/session MCP tools in `crates/ibkr-mcp/src/tools/health.rs`
- [ ] T097 [US3] Implement account and portfolio MCP tools in `crates/ibkr-mcp/src/tools/accounts.rs` and `crates/ibkr-mcp/src/tools/portfolio.rs`
- [ ] T098 [US3] Implement contract and market MCP tools in `crates/ibkr-mcp/src/tools/market.rs`
- [ ] T099 [US3] Implement read-only orders MCP tools in `crates/ibkr-mcp/src/tools/orders.rs`
- [ ] T100 [US3] Implement MCP stdio server entrypoint in `crates/ibkr-mcp/src/server.rs`
- [ ] T101 [US3] Implement optional MCP-session keepalive loop in `crates/ibkr-mcp/src/keepalive.rs`
- [ ] T102 [US3] Wire MCP crate exports in `crates/ibkr-mcp/src/lib.rs`
- [ ] T103 [US3] Add CLI `mcp serve --transport stdio` command in `crates/ibkr-cli/src/commands/mcp.rs`
- [ ] T104 [US3] Wire MCP serve command in `crates/ibkr-cli/src/main.rs`
- [ ] T105 [US3] Emit audit events for MCP broker tool calls, denials, refusals, failures, and completions using shared recorder in `crates/ibkr-mcp/src/audit.rs`
- [ ] T106 [US3] Document local MCP setup and broker tool list in `docs/mcp-local.md`
- [ ] T107 [US3] Document scope-to-tool mapping in `docs/scopes.md`

**Checkpoint**: US3 can be validated with MCP tool-list/schema tests and local stdio serving without remote OAuth, sidecar, provider-specific SDKs, or public networking.

---

## Phase 6: User Story 4 - Audit Read-Only Activity (Priority: P4)

**Goal**: A user can review allowed, denied, refused, failed, and completed read-only activity with correlation, redaction, and HMAC account hashing.

**Independent Test**: Allowed and denied CLI/MCP operations produce audit events that reconstruct tool, scope, account hash, decision, status, and redaction metadata without storing secrets.

### Tests for User Story 4

- [ ] T108 [P] [US4] Add audit event contract tests for required event shapes in `tests/contract_audit_events.rs`
- [ ] T109 [P] [US4] Add account hash and redaction replay tests in `tests/replay_audit_redaction.rs`
- [ ] T110 [P] [US4] Add audit tail CLI contract tests in `tests/contract_cli_audit_tail.rs`
- [ ] T111 [P] [US4] Add audit tail MCP contract tests in `tests/contract_mcp_audit_tail.rs`
- [ ] T112 [P] [US4] Add audit read scope denial tests in `tests/integration_audit_scope_denials.rs`

### Implementation for User Story 4

- [ ] T113 [US4] Implement audit event query and tail methods in `crates/ibkr-audit/src/sqlite.rs`
- [ ] T114 [US4] Implement audit tail domain/query DTOs in `crates/ibkr-audit/src/query.rs`
- [ ] T115 [US4] Wire audit query exports in `crates/ibkr-audit/src/lib.rs`
- [ ] T116 [US4] Implement CLI audit tail output in `crates/ibkr-cli/src/commands/audit.rs`
- [ ] T117 [US4] Implement MCP audit tail handler in `crates/ibkr-mcp/src/tools/audit.rs`
- [ ] T118 [US4] Add `ibkr_audit_tail` to MCP registry only after US4 in `crates/ibkr-mcp/src/registry.rs`
- [ ] T119 [US4] Wire audit CLI command in `crates/ibkr-cli/src/main.rs`
- [ ] T120 [US4] Document audit storage, HMAC redaction, and review flow in `docs/audit-log.md`

**Checkpoint**: US4 can be validated by running allowed and denied operations, then reviewing CLI and MCP audit tail output.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, docs, CI, replay, and security checks across all completed read-only stories.

- [ ] T121 [P] Add CI workflow for fmt, clippy, test, and docs checks in `.github/workflows/ci.yml`
- [ ] T122 [P] Add README project overview and quickstart links in `README.md`
- [ ] T123 [P] Complete local setup documentation in `docs/getting-started-local.md`
- [ ] T124 [P] Complete broker session troubleshooting documentation in `docs/ibkr-client-portal-gateway.md`
- [ ] T125 [P] Complete fixture and replay testing documentation in `docs/testing.md`
- [ ] T126 [P] Complete scope and local-auth documentation in `docs/scopes.md`
- [ ] T127 Add secret scanning assertions for fixture outputs in `tests/replay_secret_scan.rs`
- [ ] T128 Add quickstart command validation coverage in `tests/integration_quickstart_readonly.rs`
- [ ] T129 Run `cargo fmt --check` and fix formatting in `Cargo.toml` and `crates/`
- [ ] T130 Run `cargo clippy --workspace --all-targets` and fix warnings in `crates/`
- [ ] T131 Run `cargo test --workspace` and fix failing tests in `crates/` and `tests/`
- [ ] T132 Review contract coverage against `specs/001-gateway-mvp-spec/contracts/` and update any missing tests in `tests/contract_mcp_schemas.rs`
- [ ] T133 Add latency budget assertions for fake backend read-only calls and audit writes in `tests/integration_performance_budgets.rs`
- [ ] T134 Add offline fixture test-suite duration measurement guidance in `docs/testing.md`
- [ ] T135 Verify no later-feature crates are created or referenced accidentally in `Cargo.toml`
- [ ] T136 Verify no forbidden tool names appear in the MCP discovery snapshot in `tests/contract_mcp_broker_tool_list.rs`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 Setup**: No dependencies.
- **Phase 2 Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3 US1**: Depends on Phase 2.
- **Phase 4 US2**: Depends on Phase 2 and can begin after US1 backend factory patterns exist; it must still be independently testable with fake fixtures.
- **Phase 5 US3**: Depends on Phase 2 plus read services from US1 and US2.
- **Phase 6 US4**: Depends on Phase 2 and operation paths from US1-US3. Record-only audit exists before US1; query/tail is added here.
- **Phase 7 Polish**: Depends on all targeted user stories.

### User Story Dependencies

- **US1 Verify Broker Session and Accounts**: MVP slice; no story dependency after foundational work.
- **US2 Inspect Portfolio and Market Data Read-Only**: Uses US1 account/session patterns but remains independently testable with fixtures.
- **US3 Use Read-Only MCP Tools from Local Agents**: Uses US1 and US2 read services and scope/audit primitives.
- **US4 Audit Read-Only Activity**: Depends on operation paths from US1-US3 for full end-to-end evidence.

### Within Each User Story

- Write story tests first and confirm they fail.
- Implement models and mappings before backend services.
- Implement backend services before CLI or MCP surfaces.
- Use the shared audit recorder with each operation path.
- Validate each story through its independent test criteria before moving on.

---

## Parallel Execution Examples

### User Story 1

```bash
# Parallel test tasks:
T043 tests/integration_backend_status.rs
T044 tests/integration_accounts_list.rs
T045 tests/integration_keepalive.rs
T046 tests/contract_cli_us1.rs
T047 tests/integration_audit_us1.rs

# Parallel implementation tasks after foundational models:
T048 crates/ibkr-cpapi/src/models.rs
T054 crates/ibkr-cli/src/commands/health.rs
T055 crates/ibkr-cli/src/commands/backend.rs
T056 crates/ibkr-cli/src/commands/accounts.rs
```

### User Story 2

```bash
# Parallel fixture and test tasks:
T061 tests/fixtures/cpapi/portfolio_snapshot.json
T062 tests/fixtures/cpapi/contracts_search_stock_etf.json
T063 tests/fixtures/cpapi/orders_list.json
T065 tests/integration_portfolio_positions.rs
T066 tests/integration_contracts_market.rs
T067 tests/integration_orders_readonly.rs

# Parallel command surface tasks after backend methods:
T078 crates/ibkr-cli/src/commands/account.rs
T079 crates/ibkr-cli/src/commands/positions.rs
T080 crates/ibkr-cli/src/commands/contracts.rs
T081 crates/ibkr-cli/src/commands/market.rs
T082 crates/ibkr-cli/src/commands/orders.rs
```

### User Story 3

```bash
# Parallel MCP contract tests:
T087 tests/contract_mcp_broker_tool_list.rs
T088 tests/contract_mcp_schemas.rs
T089 tests/integration_mcp_scope_denials.rs
T090 tests/integration_mcp_write_refusals.rs
T091 tests/integration_mcp_redaction.rs

# Parallel tool handlers after registry and scope guard:
T096 crates/ibkr-mcp/src/tools/health.rs
T097 crates/ibkr-mcp/src/tools/accounts.rs
T098 crates/ibkr-mcp/src/tools/market.rs
T099 crates/ibkr-mcp/src/tools/orders.rs
```

### User Story 4

```bash
# Parallel audit tests:
T108 tests/contract_audit_events.rs
T109 tests/replay_audit_redaction.rs
T110 tests/contract_cli_audit_tail.rs
T111 tests/contract_mcp_audit_tail.rs
T112 tests/integration_audit_scope_denials.rs
```

---

## Implementation Strategy

### MVP First

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 only.
3. Validate `ibkr-agent health`, `ibkr-agent backend status`, `ibkr-agent session requirements`, and `ibkr-agent accounts list` using fake connected, missing-session, expired-session, and keepalive fixtures.
4. Stop and review before expanding to portfolio, market data, MCP, and audit tail.

### Incremental Delivery

1. US1: session, keepalive, and accounts read-only.
2. US2: portfolio, market data, and read-only orders.
3. US3: MCP local read-only broker exposure.
4. US4: audit review and tailing.
5. Polish: CI, docs, replay, quickstart validation, performance checks, and secret scans.

### Team Parallel Strategy

After Phase 2, one developer can extend CPAPI/backend read flows for US2 while another prepares MCP schema/tool-list tests for US3 and another hardens audit query/tail for US4. Keep file ownership split by crate or top-level test file to avoid conflicts.

---

## Notes

- [P] tasks are parallelizable only when their listed files do not overlap.
- Every story includes tests because this feature touches broker access, scopes, MCP surfaces, and audit behavior.
- Do not add order preview, submit, cancel, modify, approve, sidecar, remote public MCP, direct broker OAuth2, provider-specific LLM clients, or live trading behavior while completing this task list.
