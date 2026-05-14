# Research: IBKR Agent Gateway Complete Roadmap

## Decision: Start from zero but keep a clean-room boundary

**Rationale**: The project owner wants to understand all internals. Starting from zero is valid, but the implementation should avoid copying official IBKR client source code and should document all protocol/API assumptions.

**Alternatives considered**:

- Fork an existing Rust wrapper: faster, less learning, less ownership.
- Translate official clients: risky licensing and poor architecture fit.

## Decision: MCP is the primary agent boundary

**Rationale**: MCP keeps the gateway provider-neutral. OpenAI, Anthropic, Cursor, Continue, and local clients can consume the same tools.

**Alternatives considered**:

- Direct OpenAI/Anthropic SDK integration first: useful for demos but couples broker logic to providers.
- Custom REST API only: simpler but loses agent/tool interoperability.

## Decision: Separate MCP authorization from IBKR authentication

**Rationale**: A user or model calling MCP tools is not the same thing as a broker session. Client authorization must be scoped and auditable independently from the broker backend auth mode.

**Alternatives considered**:

- Treat local broker session as authorization: unsafe for remote tools.
- Give provider clients broker tokens: unacceptable secret exposure.

## Decision: Use Client Portal Gateway for the first backend

**Rationale**: It is the practical starting point for a local retail/individual workflow. It also forces a realistic sidecar design later.

**Alternatives considered**:

- Direct IBKR OAuth2 first: cleaner when available, but not universally available.
- TWS/IB Gateway first: powerful, but not the selected Web API/Client Portal direction.

## Decision: Use `IbkrBackend` abstraction immediately

**Rationale**: It enables fake fixtures, future direct OAuth2 backend, optional TWS backend, and sidecar routing without rewriting MCP or domain layers.

**Alternatives considered**:

- CPAPI calls directly in MCP handlers: fast but brittle.

## Decision: Audit is a foundation, not a later add-on

**Rationale**: Once agents can read broker data, the user needs to know what was read, denied, failed, or redacted. Write-capable features require audit even more.

**Alternatives considered**:

- Add audit only when trading is enabled: too late and harder to retrofit.

## Decision: HMAC identifiers by default

**Rationale**: Broker account identifiers are low-entropy enough that raw SHA-256 can be brute-forced. HMAC-SHA256 with a local/deployment secret is the default.

**Alternatives considered**:

- Store raw account ids: easier debugging, worse privacy.
- SHA-256 only: deterministic but weak against enumeration.

## Decision: Order features are split into preview, paper, and live

**Rationale**: Preview validates intent and risk without execution. Paper validates broker write mechanics without live capital. Live adds final gates.

**Alternatives considered**:

- Add preview and submit together: too easy to blur the boundary.
- Add live trading after read-only: skips essential safety layers.

## Decision: Sidecar relay is separate from remote OAuth MCP

**Rationale**: Remote OAuth protects MCP clients. Sidecar relay solves local broker session reachability. They are different problems and must not be fused.

**Alternatives considered**:

- Run remote gateway with local broker secrets: bad operational model.
- Require all users to have direct broker OAuth2: excludes many retail workflows.

## Decision: Provider compatibility is a test layer

**Rationale**: Providers differ in MCP support details, but broker logic should not depend on provider SDKs.

**Alternatives considered**:

- Dedicated OpenAI and Anthropic broker clients: more code paths and higher drift.
