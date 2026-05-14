# Tasks: IBKR Agent Gateway Read-Only MVP

**Input**: Design documents from `/specs/001-gateway-mvp-spec/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Required for this feature. The constitution and spec require unit,
contract, integration, MCP schema/tool-list, audit redaction, scope denial, and
fixture replay coverage.

**Organization**: Tasks are grouped by user story to enable independent
implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel because it touches different files and does not
  depend on an incomplete task.
- **[Story]**: User story label for story phases only.
- Every task includes at least one exact file path.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the Rust workspace, crate boundaries, toolchain pin, and
baseline documentation required by every story.

- [ ] T001 Create Rust workspace manifest with member crates in `Cargo.toml`
- [ ] T002 Pin Rust 1.94.1 and edition defaults in `rust-toolchain.toml` and `Cargo.toml`
- [ ] T003 [P] Add shared Cargo lint and test aliases in `.cargo/config.toml`
- [ ] T004 [P] Create crate manifests for `crates/ibkr-domain/Cargo.toml`, `crates/ibkr-backend/Cargo.toml`, `crates/ibkr-cpapi/Cargo.toml`, `crates/ibkr-auth/Cargo.toml`, `crates/ibkr-audit/Cargo.toml`, `crates/ibkr-mcp/Cargo.toml`, and `crates/ibkr-cli/Cargo.toml`
- [ ] T005 [P] Create initial crate entrypoints in `crates/ibkr-domain/src/lib.rs`, `crates/ibkr-backend/src/lib.rs`, `crates/ibkr-cpapi/src/lib.rs`, `crates/ibkr-auth/src/lib.rs`, `crates/ibkr-audit/src/lib.rs`, `crates/ibkr-mcp/src/lib.rs`, and `crates/ibkr-cli/src/main.rs`
- [ ] T006 [P] Create repository test directories in `tests/fixtures/cpapi/`, `tests/contract-tests/`, `tests/integration/`, and `tests/replay/`
- [ ] T007 [P] Add local config example matching the config contract in `config/local.example.yaml`
- [ ] T008 [P] Add getting-started documentation scaffold in `docs/getting-started-local.md`
- [ ] T009 [P] Add broker gateway documentation scaffold in `docs/ibkr-client-portal-gateway.md`
- [ ] T010 Verify workspace bootstraps with empty crates by running checks documented in `specs/001-gateway-mvp-spec/quickstart.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement shared domain, errors, configuration, scopes, backend
traits, and audit primitives that all user stories depend on.

**Critical**: No user story work can begin until this phase is complete.

- [ ] T011 [P] Implement typed identifiers and request correlation types in `crates/ibkr-domain/src/identifiers.rs`
- [ ] T012 [P] Implement decimal-safe money and quantity types in `crates/ibkr-domain/src/money.rs`
- [ ] T013 [P] Implement shared typed error codes and user actions in `crates/ibkr-domain/src/error.rs`
- [ ] T014 [P] Implement broker account and session status models in `crates/ibkr-domain/src/account.rs`
- [ ] T015 [P] Implement contract and market data models in `crates/ibkr-domain/src/contract.rs` and `crates/ibkr-domain/src/market.rs`
- [ ] T016 [P] Implement read-only order record models and forbid write states in `crates/ibkr-domain/src/order.rs`
- [ ] T017 Wire all domain modules and serde/schemars exports in `crates/ibkr-domain/src/lib.rs`
- [ ] T018 [P] Implement gateway configuration models and read-only validation in `crates/ibkr-domain/src/config.rs`
- [ ] T019 [P] Implement local scope constants and scope-set validation in `crates/ibkr-auth/src/scopes.rs`
- [ ] T020 [P] Implement local user identity and scope-check result types in `crates/ibkr-auth/src/local_user.rs`
- [ ] T021 Implement auth crate exports and deny-by-default helpers in `crates/ibkr-auth/src/lib.rs`
- [ ] T022 [P] Implement `IbkrBackend` read-only trait in `crates/ibkr-backend/src/trait.rs`
- [ ] T023 [P] Implement fake backend fixture loader in `crates/ibkr-backend/src/fake.rs`
- [ ] T024 Wire backend crate exports in `crates/ibkr-backend/src/lib.rs`
- [ ] T025 [P] Implement audit event model and required event types in `crates/ibkr-audit/src/event.rs`
- [ ] T026 [P] Implement audit redaction and stable hashing helpers in `crates/ibkr-audit/src/redaction.rs`
- [ ] T027 Implement SQLite audit writer and migrations in `crates/ibkr-audit/src/sqlite.rs` and `crates/ibkr-audit/migrations/0001_audit_events.sql`
- [ ] T028 Wire audit crate exports in `crates/ibkr-audit/src/lib.rs`
- [ ] T029 [P] Add unit tests for domain validation in `crates/ibkr-domain/src/lib.rs`
- [ ] T030 [P] Add unit tests for scope validation in `crates/ibkr-auth/src/lib.rs`
- [ ] T031 [P] Add unit tests for audit redaction in `crates/ibkr-audit/src/lib.rs`
- [ ] T032 Run foundational checks with `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and `cargo test --workspace` from `Cargo.toml`

**Checkpoint**: Domain, scopes, backend trait, config, and audit primitives are
ready for story implementation.

---

## Phase 3: User Story 1 - Verify Broker Session and Accounts (Priority: P1)

**Goal**: A local user can verify broker session usability and list accessible
accounts without leaking secrets.

**Independent Test**: With fake connected and missing-session fixtures, status
and account discovery return either safe account metadata or a manual-action
required refusal.

### Tests for User Story 1

- [ ] T033 [P] [US1] Add fake connected account fixture in `tests/fixtures/cpapi/accounts_success.json`
- [ ] T034 [P] [US1] Add fake missing-session fixture in `tests/fixtures/cpapi/session_required.json`
- [ ] T035 [P] [US1] Add backend status integration tests in `tests/integration/backend_status.rs`
- [ ] T036 [P] [US1] Add account list integration tests in `tests/integration/accounts_list.rs`
- [ ] T037 [P] [US1] Add CLI contract tests for health, backend status, session requirements, and accounts list in `tests/contract-tests/cli_us1.rs`
- [ ] T038 [P] [US1] Add audit/security tests for session and account errors in `tests/integration/audit_us1.rs`

### Implementation for User Story 1

- [ ] T039 [P] [US1] Implement Client Portal response models for session and accounts in `crates/ibkr-cpapi/src/models.rs`
- [ ] T040 [US1] Implement Client Portal HTTP client health, session, and accounts calls in `crates/ibkr-cpapi/src/client.rs`
- [ ] T041 [US1] Implement CPAPI session/account error mapping in `crates/ibkr-cpapi/src/mapper.rs`
- [ ] T042 [US1] Wire CPAPI crate exports in `crates/ibkr-cpapi/src/lib.rs`
- [ ] T043 [US1] Implement `ClientPortalBackend` health and list accounts in `crates/ibkr-backend/src/client_portal.rs`
- [ ] T044 [US1] Add backend factory for client portal and fake backend in `crates/ibkr-backend/src/factory.rs`
- [ ] T045 [US1] Implement CLI health command in `crates/ibkr-cli/src/commands/health.rs`
- [ ] T046 [US1] Implement CLI backend status and session requirements commands in `crates/ibkr-cli/src/commands/backend.rs`
- [ ] T047 [US1] Implement CLI accounts list command in `crates/ibkr-cli/src/commands/accounts.rs`
- [ ] T048 [US1] Implement shared CLI JSON and human output formatting in `crates/ibkr-cli/src/output.rs`
- [ ] T049 [US1] Wire CLI command routing for US1 commands in `crates/ibkr-cli/src/main.rs`
- [ ] T050 [US1] Emit audit events for health, backend status, session requirements, and accounts list in `crates/ibkr-cli/src/audit.rs`
- [ ] T051 [US1] Document US1 commands and expected missing-session behavior in `docs/getting-started-local.md`

**Checkpoint**: US1 can be validated independently with CLI commands and fake
fixtures before portfolio, market data, MCP, or audit tail work exists.

---

## Phase 4: User Story 2 - Inspect Portfolio and Market Data Read-Only (Priority: P2)

**Goal**: A local user can inspect account summary, positions, portfolio
snapshot, contract candidates, market data, read-only orders, and executions
without any write capability.

**Independent Test**: Offline fixtures return structured read-only data or clear
refusals for missing account, ambiguous contract, stale market data, and
write-like requests.

### Tests for User Story 2

- [ ] T052 [P] [US2] Add portfolio and positions fixtures in `tests/fixtures/cpapi/portfolio_snapshot.json` and `tests/fixtures/cpapi/positions_list.json`
- [ ] T053 [P] [US2] Add contract and market data fixtures in `tests/fixtures/cpapi/contracts_search.json`, `tests/fixtures/cpapi/contracts_ambiguous.json`, and `tests/fixtures/cpapi/market_snapshot.json`
- [ ] T054 [P] [US2] Add orders and executions fixtures in `tests/fixtures/cpapi/orders_list.json`, `tests/fixtures/cpapi/order_status.json`, and `tests/fixtures/cpapi/executions_list.json`
- [ ] T055 [P] [US2] Add account context refusal tests in `tests/integration/account_context_refusals.rs`
- [ ] T056 [P] [US2] Add portfolio and positions integration tests in `tests/integration/portfolio_positions.rs`
- [ ] T057 [P] [US2] Add contract ambiguity and market snapshot integration tests in `tests/integration/contracts_market.rs`
- [ ] T058 [P] [US2] Add read-only orders integration tests in `tests/integration/orders_readonly.rs`
- [ ] T059 [P] [US2] Add forbidden write refusal tests in `tests/integration/write_refusals.rs`
- [ ] T060 [P] [US2] Add CLI contract tests for portfolio, market, and orders commands in `tests/contract-tests/cli_us2.rs`

### Implementation for User Story 2

- [ ] T061 [P] [US2] Implement CPAPI portfolio, positions, and summary response models in `crates/ibkr-cpapi/src/models.rs`
- [ ] T062 [P] [US2] Implement CPAPI contract, market data, orders, and executions response models in `crates/ibkr-cpapi/src/models.rs`
- [ ] T063 [US2] Implement CPAPI read calls for account summary, positions, portfolio, contracts, market data, orders, and executions in `crates/ibkr-cpapi/src/client.rs`
- [ ] T064 [US2] Implement CPAPI read data mapping and ambiguity detection in `crates/ibkr-cpapi/src/mapper.rs`
- [ ] T065 [US2] Implement backend trait methods for account summary, positions, portfolio snapshot, contract search, contract resolve, market snapshot, historical bars, orders list, order status, and executions list in `crates/ibkr-backend/src/client_portal.rs`
- [ ] T066 [US2] Extend fake backend for all US2 read-only flows in `crates/ibkr-backend/src/fake.rs`
- [ ] T067 [US2] Implement account context validation helper in `crates/ibkr-backend/src/account_context.rs`
- [ ] T068 [US2] Implement CLI account summary and portfolio commands in `crates/ibkr-cli/src/commands/account.rs` and `crates/ibkr-cli/src/commands/portfolio.rs`
- [ ] T069 [US2] Implement CLI positions command in `crates/ibkr-cli/src/commands/positions.rs`
- [ ] T070 [US2] Implement CLI contract commands in `crates/ibkr-cli/src/commands/contracts.rs`
- [ ] T071 [US2] Implement CLI market data commands in `crates/ibkr-cli/src/commands/market.rs`
- [ ] T072 [US2] Implement CLI read-only orders and executions commands in `crates/ibkr-cli/src/commands/orders.rs`
- [ ] T073 [US2] Add explicit forbidden write command refusals in `crates/ibkr-cli/src/commands/orders.rs`
- [ ] T074 [US2] Wire US2 CLI commands in `crates/ibkr-cli/src/main.rs`
- [ ] T075 [US2] Emit audit events for portfolio, market, order-read, ambiguity, stale-data, and write-refusal paths in `crates/ibkr-cli/src/audit.rs`
- [ ] T076 [US2] Document read-only data commands and write refusal behavior in `docs/tools.md`

**Checkpoint**: US2 can be validated independently through CLI plus fake
fixtures and must still expose no write path.

---

## Phase 5: User Story 3 - Use Read-Only MCP Tools from Local Agents (Priority: P3)

**Goal**: A local MCP-compatible client can list and call only read-only IBKR
tools with strict schemas, minimal scopes, and safe outputs.

**Independent Test**: A local MCP client or tool-list test sees only read-only
tools; missing scopes are denied; forbidden order preview/submit/cancel tools
are absent or refused.

### Tests for User Story 3

- [ ] T077 [P] [US3] Add MCP tool list snapshot test in `tests/contract-tests/mcp_tool_list.rs`
- [ ] T078 [P] [US3] Add MCP schema snapshot tests for all read-only tools in `tests/contract-tests/mcp_schemas.rs`
- [ ] T079 [P] [US3] Add MCP missing-scope denial tests in `tests/integration/mcp_scope_denials.rs`
- [ ] T080 [P] [US3] Add MCP forbidden write tool absence/refusal tests in `tests/integration/mcp_write_refusals.rs`
- [ ] T081 [P] [US3] Add MCP safe-output redaction tests in `tests/integration/mcp_redaction.rs`

### Implementation for User Story 3

- [ ] T082 [P] [US3] Implement MCP request and response schemas from domain models in `crates/ibkr-mcp/src/schemas.rs`
- [ ] T083 [US3] Implement MCP tool registry with only read-only tool names in `crates/ibkr-mcp/src/registry.rs`
- [ ] T084 [US3] Implement MCP scope enforcement middleware in `crates/ibkr-mcp/src/scope_guard.rs`
- [ ] T085 [US3] Implement health/session MCP tools in `crates/ibkr-mcp/src/tools/health.rs`
- [ ] T086 [US3] Implement account and portfolio MCP tools in `crates/ibkr-mcp/src/tools/accounts.rs` and `crates/ibkr-mcp/src/tools/portfolio.rs`
- [ ] T087 [US3] Implement contract and market MCP tools in `crates/ibkr-mcp/src/tools/market.rs`
- [ ] T088 [US3] Implement read-only orders MCP tools in `crates/ibkr-mcp/src/tools/orders.rs`
- [ ] T089 [US3] Implement audit tail MCP tool in `crates/ibkr-mcp/src/tools/audit.rs`
- [ ] T090 [US3] Implement MCP stdio server entrypoint in `crates/ibkr-mcp/src/server.rs`
- [ ] T091 [US3] Wire MCP crate exports in `crates/ibkr-mcp/src/lib.rs`
- [ ] T092 [US3] Add CLI `mcp serve --transport stdio` command in `crates/ibkr-cli/src/commands/mcp.rs`
- [ ] T093 [US3] Wire MCP serve command in `crates/ibkr-cli/src/main.rs`
- [ ] T094 [US3] Document local MCP setup and tool list in `docs/mcp-local.md`
- [ ] T095 [US3] Document scope-to-tool mapping in `docs/scopes.md`

**Checkpoint**: US3 can be validated with MCP tool-list/schema tests and local
stdio serving without remote OAuth or public networking.

---

## Phase 6: User Story 4 - Audit Read-Only Activity (Priority: P4)

**Goal**: A user can review allowed and denied read-only activity with
correlation, redaction, and stable account hashing.

**Independent Test**: Allowed and denied CLI/MCP operations produce audit events
that reconstruct tool, scope, account hash, decision, status, and redaction
metadata without storing secrets.

### Tests for User Story 4

- [ ] T096 [P] [US4] Add audit event contract tests for required event shapes in `tests/contract-tests/audit_events.rs`
- [ ] T097 [P] [US4] Add SQLite audit persistence tests in `tests/integration/audit_sqlite.rs`
- [ ] T098 [P] [US4] Add account hash and redaction replay tests in `tests/replay/audit_redaction_replay.rs`
- [ ] T099 [P] [US4] Add audit tail CLI contract tests in `tests/contract-tests/cli_audit_tail.rs`
- [ ] T100 [P] [US4] Add audit tail MCP contract tests in `tests/contract-tests/mcp_audit_tail.rs`

### Implementation for User Story 4

- [ ] T101 [US4] Implement stable account hashing policy in `crates/ibkr-audit/src/redaction.rs`
- [ ] T102 [US4] Implement audit event query and tail methods in `crates/ibkr-audit/src/sqlite.rs`
- [ ] T103 [US4] Implement shared audit recorder service in `crates/ibkr-audit/src/recorder.rs`
- [ ] T104 [US4] Wire audit recorder exports in `crates/ibkr-audit/src/lib.rs`
- [ ] T105 [US4] Replace story-local audit helpers with shared recorder calls in `crates/ibkr-cli/src/audit.rs`
- [ ] T106 [US4] Replace MCP audit helper calls with shared recorder calls in `crates/ibkr-mcp/src/audit.rs`
- [ ] T107 [US4] Implement CLI audit tail output in `crates/ibkr-cli/src/commands/audit.rs`
- [ ] T108 [US4] Implement MCP audit tail handler using shared recorder in `crates/ibkr-mcp/src/tools/audit.rs`
- [ ] T109 [US4] Document audit storage, redaction, and review flow in `docs/audit-log.md`

**Checkpoint**: US4 can be validated by running allowed and denied operations,
then reviewing CLI and MCP audit tail output.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final validation, docs, CI, replay, and security checks across all
completed read-only stories.

- [ ] T110 [P] Add CI workflow for fmt, clippy, test, and docs checks in `.github/workflows/ci.yml`
- [ ] T111 [P] Add README project overview and quickstart links in `README.md`
- [ ] T112 [P] Complete local setup documentation in `docs/getting-started-local.md`
- [ ] T113 [P] Complete broker session troubleshooting documentation in `docs/ibkr-client-portal-gateway.md`
- [ ] T114 [P] Complete fixture and replay testing documentation in `docs/testing.md`
- [ ] T115 Add secret scanning assertions for fixture outputs in `tests/replay/secret_scan.rs`
- [ ] T116 Add quickstart command validation coverage in `tests/integration/quickstart_readonly.rs`
- [ ] T117 Run `cargo fmt --check` and fix formatting in `Cargo.toml` and `crates/`
- [ ] T118 Run `cargo clippy --workspace --all-targets` and fix warnings in `crates/`
- [ ] T119 Run `cargo test --workspace` and fix failing tests in `crates/` and `tests/`
- [ ] T120 Review contract coverage against `specs/001-gateway-mvp-spec/contracts/` and update any missing tests in `tests/contract-tests/`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 Setup**: No dependencies.
- **Phase 2 Foundational**: Depends on Phase 1 and blocks all user stories.
- **Phase 3 US1**: Depends on Phase 2.
- **Phase 4 US2**: Depends on Phase 2 and can begin after US1 backend factory
  patterns exist; it must still be independently testable with fake fixtures.
- **Phase 5 US3**: Depends on Phase 2 plus read services from US1 and US2.
- **Phase 6 US4**: Depends on Phase 2 and can be refined after US1-US3 emit
  operation events.
- **Phase 7 Polish**: Depends on all targeted user stories.

### User Story Dependencies

- **US1 Verify Broker Session and Accounts**: MVP slice; no story dependency
  after foundational work.
- **US2 Inspect Portfolio and Market Data Read-Only**: Uses US1 account/session
  patterns but remains independently testable with fixtures.
- **US3 Use Read-Only MCP Tools from Local Agents**: Uses US1 and US2 read
  services and scope primitives.
- **US4 Audit Read-Only Activity**: Depends on operation paths from US1-US3 for
  full end-to-end evidence.

### Within Each User Story

- Write story tests first and confirm they fail.
- Implement models and mappings before backend services.
- Implement backend services before CLI or MCP surfaces.
- Add audit emission with each operation path.
- Validate each story through its independent test criteria before moving on.

---

## Parallel Execution Examples

### User Story 1

```bash
# Parallel test tasks:
T035 tests/integration/backend_status.rs
T036 tests/integration/accounts_list.rs
T037 tests/contract-tests/cli_us1.rs
T038 tests/integration/audit_us1.rs

# Parallel implementation tasks after foundational models:
T039 crates/ibkr-cpapi/src/models.rs
T045 crates/ibkr-cli/src/commands/health.rs
T046 crates/ibkr-cli/src/commands/backend.rs
T047 crates/ibkr-cli/src/commands/accounts.rs
```

### User Story 2

```bash
# Parallel fixture and test tasks:
T052 tests/fixtures/cpapi/portfolio_snapshot.json
T053 tests/fixtures/cpapi/contracts_search.json
T054 tests/fixtures/cpapi/orders_list.json
T056 tests/integration/portfolio_positions.rs
T057 tests/integration/contracts_market.rs
T058 tests/integration/orders_readonly.rs

# Parallel command surface tasks after backend methods:
T068 crates/ibkr-cli/src/commands/account.rs
T069 crates/ibkr-cli/src/commands/positions.rs
T070 crates/ibkr-cli/src/commands/contracts.rs
T071 crates/ibkr-cli/src/commands/market.rs
T072 crates/ibkr-cli/src/commands/orders.rs
```

### User Story 3

```bash
# Parallel MCP contract tests:
T077 tests/contract-tests/mcp_tool_list.rs
T078 tests/contract-tests/mcp_schemas.rs
T079 tests/integration/mcp_scope_denials.rs
T080 tests/integration/mcp_write_refusals.rs
T081 tests/integration/mcp_redaction.rs

# Parallel tool handlers after registry and scope guard:
T085 crates/ibkr-mcp/src/tools/health.rs
T086 crates/ibkr-mcp/src/tools/accounts.rs
T087 crates/ibkr-mcp/src/tools/market.rs
T088 crates/ibkr-mcp/src/tools/orders.rs
T089 crates/ibkr-mcp/src/tools/audit.rs
```

### User Story 4

```bash
# Parallel audit tests:
T096 tests/contract-tests/audit_events.rs
T097 tests/integration/audit_sqlite.rs
T098 tests/replay/audit_redaction_replay.rs
T099 tests/contract-tests/cli_audit_tail.rs
T100 tests/contract-tests/mcp_audit_tail.rs
```

---

## Implementation Strategy

### MVP First

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 only.
3. Validate `ibkr-agent health`, `ibkr-agent backend status`,
   `ibkr-agent session requirements`, and `ibkr-agent accounts list` using fake
   connected and missing-session fixtures.
4. Stop and review before expanding to portfolio, market data, MCP, and audit
   tail.

### Incremental Delivery

1. US1: session and accounts read-only.
2. US2: portfolio, market data, and read-only orders.
3. US3: MCP local read-only exposure.
4. US4: audit review and tailing.
5. Polish: CI, docs, replay, quickstart validation, and secret checks.

### Team Parallel Strategy

After Phase 2, one developer can extend CPAPI/backend read flows for US2 while
another prepares MCP schema/tool-list tests for US3 and another hardens audit
storage for US4. Keep file ownership split by crate or test directory to avoid
conflicts.

---

## Notes

- [P] tasks are parallelizable only when their listed files do not overlap.
- Every story includes tests because this feature touches broker access, scopes,
  MCP surfaces, and audit behavior.
- Do not add order preview, submit, cancel, sidecar, remote public MCP, or live
  trading behavior while completing this task list.
