# Research: IBKR Agent Gateway Read-Only MVP

## Decision: Treat this spec as the first slice of the complete roadmap

**Rationale**: The complete project is defined in `/specs/000-project-roadmap/`. This spec implements only the local read-only foundation and must not accidentally include later features.

**Alternatives considered**:

- Put the whole product into one feature spec: too broad and impossible to validate independently.
- Keep only the MVP without a roadmap: easy to code but loses the architecture target.

## Decision: Use a Rust Cargo workspace from the first increment

**Rationale**: The project has explicit trust boundaries: pure domain types, broker adapter, backend abstraction, scope checks, MCP exposure, CLI, config, and audit. A workspace keeps these boundaries testable without mixing LLM, broker session, and persistence concerns.

**Alternatives considered**:

- Single binary crate: simpler initially, but it would blur domain, broker, MCP, config, and audit boundaries.
- Library-only crate: insufficient because the MVP must expose a local operator CLI and an MCP server.

## Decision: Pin the selected local stable toolchain during bootstrap

**Rationale**: The exact installed compiler should be recorded in `rust-toolchain.toml` once the repository is bootstrapped. The spec should not guess a compiler version.

**Alternatives considered**:

- Hard-code a guessed compiler version in the spec: can become wrong before implementation.
- Floating stable: easier but creates reproducibility drift.

## Decision: Use Client Portal Gateway as the only real broker backend in MVP

**Rationale**: The complete roadmap identifies the local Client Portal Gateway as the practical retail/individual path. Remote direct IBKR OAuth2 depends on target availability and should not block the local read-only MVP.

**Alternatives considered**:

- Direct IBKR OAuth2 first: cleaner architecture when available, but not universal.
- TWS adapter first: possible later, but it expands scope before the core Web API/Client Portal safety model exists.

## Decision: Model backend access behind an `IbkrBackend` abstraction

**Rationale**: Even though the MVP uses Client Portal Gateway only, tests need a fake backend and future phases need OAuth2, sidecar, or optional TWS backends. A small trait lets read-only flows stay deterministic and testable.

**Alternatives considered**:

- Call the Client Portal client directly from CLI/MCP: less code, but makes fixture testing and later backend replacement harder.
- Full plugin backend system: unnecessary before multiple real backends exist.

## Decision: Implement audit recorder in the foundational phase

**Rationale**: US1/US2/US3 should use the same `AuditRecorder` immediately. Temporary story-local audit helpers would create avoidable churn and inconsistent event shapes.

**Alternatives considered**:

- Add audit recorder only in US4: easier start, but requires replacing earlier code.
- Log-only audit initially: insufficient for later audit tail and replay tests.

## Decision: Audit tail belongs to US4, not US3

**Rationale**: US3 is local MCP broker-read exposure. US4 is audit review. Keeping `ibkr_audit_tail` in US4 avoids phase mismatch.

**Alternatives considered**:

- Include audit tail in MCP US3: possible, but then audit query/tail must exist before the audit story.

## Decision: SQLite append-only audit for local MVP

**Rationale**: SQLite is simple for single-user local operation and supports durable audit review without requiring a server database. Audit records can be exported as JSONL later.

**Alternatives considered**:

- JSONL-only audit: easy to inspect, weaker for querying and correlation.
- Postgres first: appropriate for remote multi-user deployments, excessive for local MVP.

## Decision: Use HMAC-SHA256 for account hash defaults

**Rationale**: Broker account IDs are low-entropy enough that raw SHA-256 can be brute-forced. HMAC with a local audit secret is the default.

**Alternatives considered**:

- Raw account IDs: easier debugging, worse privacy.
- SHA-256 only: deterministic but weak against enumeration.

## Decision: Keep runtime configuration outside `ibkr-domain`

**Rationale**: The constitution requires `ibkr-domain` to remain free of storage, HTTP, MCP, OAuth, LLM, and runtime concerns. The read-only MVP configuration contains audit storage, local bind addresses, broker base URLs, TLS validation, market-data policy, and safety flags, so it belongs in a dedicated configuration/application layer.

**Alternatives considered**:

- Put `GatewayConfiguration` in `ibkr-domain`: convenient for sharing types, but it introduces runtime concerns into the pure domain crate.
- Put config parsing only in `ibkr-cli`: MCP and tests also need the same validated config.

## Decision: Allow `verify_tls=false` only for localhost URLs

**Rationale**: Client Portal Gateway often runs locally with a certificate setup that may require TLS verification bypass in development. That bypass must not be allowed for network hosts.

**Alternatives considered**:

- Always require TLS verification: may break local CP Gateway use.
- Allow TLS bypass anywhere: unacceptable for remote/network endpoints.

## Decision: Keep historical bars in read-only MVP scope when available

**Rationale**: Historical bars are a read-only market-data capability already listed in the MCP and CLI contracts. Keeping them in scope is consistent with the provider-neutral read-only value proposition as long as availability failures return structured refusals.

**Alternatives considered**:

- Remove historical bars from contracts: simpler, but less aligned with the initial tool catalog and market-data read use case.

## Decision: Limit MVP asset classes to stocks and ETFs

**Rationale**: Options, futures, forex, bonds, CFDs, combos, and derivatives require additional contract resolution, permissions, market-data, and order semantics. Stocks/ETFs are enough to prove the read-only gateway.

**Alternatives considered**:

- Support all asset classes immediately: too broad for the first slice.
- Support stock only: simpler but ETFs are usually handled similarly enough for read-only lookup.

## Decision: Define explicit market-data stale/delayed policy

**Rationale**: Agent-visible prices without freshness/status are dangerous even in read-only workflows. The output must include `data_status`, `staleness_seconds`, and warnings/refusal according to config.

**Alternatives considered**:

- Return best-effort values only: misleading to agents.
- Always refuse delayed data: too strict for read-only analysis when the user accepts delayed labels.

## Decision: Local scope enforcement without full OAuth/OIDC

**Rationale**: The read-only local MVP must enforce per-tool scopes, but remote OAuth/OIDC front-door behavior is explicitly out of scope. A local identity and scope model can validate the authorization boundary without building fake OAuth early.

**Alternatives considered**:

- No scopes in MVP: violates the project constitution and makes later MCP exposure unsafe.
- Full OAuth/OIDC first: valuable for remote MCP, but expands the MVP beyond local read-only value.

## Decision: Use typed MCP tool schemas generated from Rust models

**Rationale**: The plan requires typed tools, strict schemas, no prompt-to-trade, and predictable validation. Schema generation from domain/request models reduces drift between code and MCP contracts.

**Alternatives considered**:

- Hand-written JSON schemas only: easy to read, but likely to drift.
- Free-form tool inputs: explicitly prohibited by the constitution.

## Decision: Use Cargo-discoverable top-level tests or explicit harnesses

**Rationale**: Nested arbitrary Rust files under `tests/` are easy to forget and may not be compiled by Cargo. The MVP should use top-level integration test files or explicit harness modules.

**Alternatives considered**:

- Keep nested `tests/integration/*.rs`: cleaner folders but easy to silently ignore unless harnesses are added.

## Decision: Use offline fake broker tests before live broker verification

**Rationale**: Broker sessions require manual login and may be unavailable in CI. Fixtures and fake Client Portal responses allow deterministic testing for accounts, positions, market data, orders read, errors, keepalive, and audit redaction.

**Alternatives considered**:

- Live-only testing: not reproducible and blocks CI.
- Unit tests only: insufficient for broker error mapping and MCP contracts.

## Decision: Fail closed on missing account, ambiguous contract, missing scope, unsupported asset class, stale data, invalid config, and unsafe output

**Rationale**: The MVP is read-only, but these checks establish the future order safety posture and prevent agents from receiving misleading account or market data.

**Alternatives considered**:

- Best-effort defaults: faster for demos, but unsafe for finance workflows.
- Ask the LLM to choose: violates the MCP boundary and deterministic domain principles.
