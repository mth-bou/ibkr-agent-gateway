# Research: IBKR Agent Gateway Read-Only MVP

## Decision: Use a Rust Cargo workspace from the first increment

**Rationale**: The project has explicit trust boundaries: pure domain types,
broker adapter, backend abstraction, scope checks, MCP exposure, CLI, and audit.
A workspace keeps these boundaries testable without mixing LLM, broker session,
and persistence concerns.

**Alternatives considered**:

- Single binary crate: simpler initially, but it would blur domain, broker, MCP,
  and audit boundaries that the constitution treats as non-negotiable.
- Library-only crate: insufficient because the MVP must expose a local operator
  CLI and an MCP server.

## Decision: Pin Rust 1.94.1 and Rust 2024 edition

**Rationale**: The local toolchain is `rustc 1.94.1` and `cargo 1.94.1`. Pinning
the toolchain avoids CI/local drift while the project is bootstrapped.

**Alternatives considered**:

- Floating stable: easier to maintain but creates reproducibility drift.
- Older MSRV-first policy: useful for libraries, but not needed before this
  local application has consumers.

## Decision: Use Client Portal Gateway as the only broker backend in MVP

**Rationale**: The source plan identifies the local Client Portal Gateway as the
practical retail/individual path. Remote direct IBKR OAuth2 depends on target
availability and should not block the local read-only MVP.

**Alternatives considered**:

- Direct IBKR OAuth2 first: cleaner architecture, but availability varies by
  target user segment.
- TWS adapter first: possible later, but it expands scope before the core read
  flows and safety model exist.

## Decision: Model backend access behind an `IbkrBackend` abstraction

**Rationale**: Even though the MVP uses Client Portal Gateway only, tests need a
fake backend and future phases need OAuth2 or sidecar backends. A small trait
lets read-only flows stay deterministic and testable.

**Alternatives considered**:

- Call the Client Portal client directly from CLI/MCP: less code, but makes
  fixture testing and later backend replacement harder.
- Full plugin backend system: unnecessary before multiple real backends exist.

## Decision: SQLite append-only audit for local MVP

**Rationale**: SQLite is simple for single-user local operation and supports
durable audit review without requiring a server database. Audit records can be
exported as JSONL later.

**Alternatives considered**:

- JSONL-only audit: easy to inspect, weaker for querying and correlation.
- Postgres first: appropriate for remote multi-user deployments, excessive for
  local MVP.

## Decision: Keep runtime configuration outside `ibkr-domain`

**Rationale**: The constitution requires `ibkr-domain` to remain free of storage,
HTTP, MCP, OAuth, LLM, and runtime concerns. The read-only MVP configuration
contains audit storage, local bind addresses, broker base URLs, and safety flags,
so it belongs in a dedicated configuration/application layer.

**Alternatives considered**:

- Put `GatewayConfiguration` in `ibkr-domain`: convenient for sharing types, but
  it introduces storage and runtime concerns into the pure domain crate.
- Put config parsing in `ibkr-cli`: acceptable for a CLI-only app, but MCP and
  tests also need the same validated config.

## Decision: Keep historical bars in read-only MVP scope when available

**Rationale**: Historical bars are a read-only market-data capability already
listed in the MCP and CLI contracts. Keeping them in scope is consistent with the
provider-neutral read-only value proposition as long as availability failures
return structured refusals.

**Alternatives considered**:

- Remove historical bars from contracts: simpler, but less aligned with the
  initial tool catalog and market-data read use case.

## Decision: Local scope enforcement without full OAuth/OIDC

**Rationale**: The read-only local MVP must enforce per-tool scopes, but remote
OAuth/OIDC front-door behavior is explicitly out of scope. A local identity and
scope model can validate the authorization boundary without building the remote
auth feature early.

**Alternatives considered**:

- No scopes in MVP: violates the constitution and would make later MCP exposure
  unsafe.
- Full OAuth/OIDC first: valuable for remote MCP, but expands the MVP beyond
  local read-only value.

## Decision: Use typed MCP tool schemas generated from Rust models

**Rationale**: The plan requires typed tools, strict schemas, no prompt-to-trade,
and predictable validation. Schema generation from domain/request models reduces
drift between code and MCP contracts.

**Alternatives considered**:

- Hand-written JSON schemas only: easy to read, but likely to drift.
- Free-form tool inputs: explicitly prohibited by the constitution.

## Decision: Use offline fake broker tests before live broker verification

**Rationale**: Broker sessions require manual login and may be unavailable in CI.
Fixtures and fake Client Portal responses allow deterministic testing for
accounts, positions, market data, orders read, errors, and audit redaction.

**Alternatives considered**:

- Live-only testing: not reproducible and blocks CI.
- Unit tests only: insufficient for broker error mapping and MCP contracts.

## Decision: Fail closed on missing account, ambiguous contract, missing scope, and unsafe output

**Rationale**: The MVP is read-only, but these checks establish the future order
safety posture and prevent agents from receiving misleading account or market
data.

**Alternatives considered**:

- Best-effort defaults: faster for demos, but unsafe for finance workflows.
- Ask the LLM to choose: violates the MCP boundary and deterministic domain
  principles.
