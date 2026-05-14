# Feature Specification: IBKR Agent Gateway Read-Only MVP

**Feature Branch**: `001-gateway-mvp-spec`

**Created**: 2026-05-14

**Status**: Draft

**Input**: First shippable feature from `/specs/000-project-roadmap/`: local, single-user, read-only IBKR Agent Gateway using Client Portal Gateway, CLI, local MCP stdio, local scopes, audit, fake backend, and fixtures.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verify Broker Session and Accounts (Priority: P1)

As a local user with an Interactive Brokers Client Portal Gateway session, I want the gateway to tell me whether the broker backend is usable and to list the accounts I am allowed to inspect, so I can confirm the system is connected before exposing any data to an agent.

**Why this priority**: Without a clear broker-session and account-read flow, no portfolio, market-data, MCP, audit, order preview, or future trading workflow can be trusted.

**Independent Test**: Can be tested with fake connected, fake missing-session, fake expired-session, and fake backend-error fixtures. Connected cases return available accounts without secrets. Missing/expired cases return clear manual action required messages and audit events.

**Acceptance Scenarios**:

1. **Given** a local broker gateway session is active, **When** the user checks gateway status and requests account discovery, **Then** the system reports a usable backend and returns only account identifiers and safe account metadata.
2. **Given** no broker gateway session is active or the session is expired, **When** the user checks gateway status or requests accounts, **Then** the system refuses broker data access and explains the manual session action required.
3. **Given** the configured backend is unavailable, rate-limited, or returns an unmapped error, **When** the user requests account discovery, **Then** the system returns a typed, non-secret error that can be audited and retried or acted on as appropriate.
4. **Given** the gateway is running for long enough that the broker session may expire, **When** keepalive/tickle fails, **Then** the system updates broker session status, records an audit event, and denies broker-backed tools until manual action restores the session.

---

### User Story 2 - Inspect Portfolio and Market Data Read-Only (Priority: P2)

As a local user, I want an agent or CLI session to read portfolio summaries, positions, supported contract candidates, market snapshots, historical bars, and recent orders without write permissions, so I can ask questions about my account without risking an order being created.

**Why this priority**: Read-only portfolio and market inspection is the first valuable agent workflow and establishes the safety boundary before any order preview or execution work.

**Independent Test**: Can be tested using offline broker fixtures and, optionally, a connected paper or live account. Each supported read-only action returns structured data or a structured refusal. No write action is exposed or accepted.

**Acceptance Scenarios**:

1. **Given** the user has selected one accessible account, **When** positions and portfolio summary are requested, **Then** the system returns structured balances, allocations, and positions for that account only.
2. **Given** the user has multiple accessible accounts and does not select one, **When** account-specific data is requested, **Then** the system refuses instead of guessing.
3. **Given** a contract lookup query is ambiguous, **When** the user searches for an instrument, **Then** the system returns candidate matches or a clear ambiguity refusal rather than selecting one silently.
4. **Given** an unsupported asset class is requested in the MVP, **When** contract search, market data, or historical bars are requested, **Then** the system returns `INPUT_UNSUPPORTED_ASSET_CLASS`.
5. **Given** market data is delayed, stale, incomplete, or unavailable, **When** snapshot or historical bars are requested, **Then** the result is labeled with data status and warnings or refused according to the configured market-data policy.
6. **Given** the user or agent attempts to submit, cancel, modify, approve, or preview an order in the read-only MVP, **When** the request reaches the gateway, **Then** the system refuses it because write tools are unavailable in this feature.

---

### User Story 3 - Use Read-Only MCP Tools from Local Agents (Priority: P3)

As a local agent user, I want supported MCP clients to discover and call only the read-only IBKR tools, so I can use agent workflows across compatible clients without coupling the broker gateway to one provider.

**Why this priority**: MCP is the provider-neutral integration boundary, but it depends on the broker-read, local scope, redaction, and audit foundations from earlier stories.

**Independent Test**: Can be tested with an MCP-compatible local client or MCP tool-list test that lists tools, calls read-only tools, confirms no write tool is present, verifies missing-scope denial, and verifies redacted output.

**Acceptance Scenarios**:

1. **Given** a local MCP client connects to the gateway over stdio, **When** it lists available tools, **Then** it sees only health/session, account, portfolio, position, contract, market-data, and read-only order-status capabilities.
2. **Given** an MCP client lacks a required local read scope, **When** it calls a protected read tool, **Then** the system denies the call and records the denied scope without leaking credentials or backend session material.
3. **Given** an MCP tool returns broker data, **When** the result is sent back to the agent context, **Then** the result excludes credentials, cookies, private backend headers, local filesystem paths, and other secrets.
4. **Given** an MCP client tries to call a forbidden write tool name, **When** dispatch receives the call, **Then** the system returns a typed read-only refusal and emits an audit event even if that tool is absent from discovery.

---

### User Story 4 - Audit Read-Only Activity (Priority: P4)

As a user reviewing gateway behavior, I want significant CLI and MCP activity to be auditable, so I can understand what an agent read, which account it touched, and why a request was allowed or refused.

**Why this priority**: Audit is required before expanding from read-only access to order previews, approvals, paper trading, remote MCP, sidecar relay, or live trading.

**Independent Test**: Can be tested by performing allowed and denied read-only operations, then reviewing CLI and MCP audit tail output for correlated, redacted events.

**Acceptance Scenarios**:

1. **Given** a read-only CLI or MCP tool call succeeds, **When** the audit log is reviewed, **Then** it includes the tool name, user/session correlation, account correlation where applicable, scope used, decision, timestamp, and result status.
2. **Given** a request is denied due to missing scope, unavailable backend, missing account, ambiguous contract, stale data, unsupported asset class, or forbidden write attempt, **When** the audit log is reviewed, **Then** it records the denial reason without storing secrets.
3. **Given** a sensitive input or output field exists, **When** the audit event is written, **Then** the field is redacted or represented by HMAC/canonical hash instead of raw secret material.
4. **Given** the audit tail is requested through CLI or MCP, **When** results are returned, **Then** they contain only redacted audit records and require `ibkr:audit:read`.

### Edge Cases

- Broker session is absent, expired, unusable, or requires manual reauthentication.
- Keepalive/tickle succeeds, fails, or reports the session has expired while the MCP server is running.
- User has multiple accounts and does not specify which account to inspect.
- Requested account is not available to the current user/session.
- Contract search returns multiple plausible instruments for the same symbol.
- Asset class is unsupported in MVP. Supported MVP asset classes are `stock` and `etf` only.
- Market data is unavailable, delayed, stale, missing timestamps, or missing currency context.
- Historical bars are unavailable for the requested contract, duration, bar size, or asset class.
- Broker backend returns partial data, rate limits, or a transient error.
- MCP client attempts a write operation even though the feature is read-only.
- External broker-provided text contains prompt-injection style content.
- Error handling or audit logging receives tokens, cookies, headers, local paths, or backend session material that must not be exposed.
- Configuration attempts to enable remote public MCP, sidecar relay, write tools, live trading, or `verify_tls=false` against a non-localhost URL.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST expose a local gateway status flow that distinguishes usable broker session, unavailable backend, expired/missing session, and manual user action required.
- **FR-002**: System MUST list accessible broker accounts with safe metadata limited to account identifier, optional display label, account mode, and base currency, without exposing credentials, cookies, backend session secrets, raw headers, or local secret paths.
- **FR-003**: System MUST allow read-only retrieval of account summary, positions, portfolio snapshot, supported contract candidates, market snapshot, historical bars when available, recent orders, order status, and executions for an explicitly selected account when the current user/session is allowed to inspect it.
- **FR-004**: System MUST refuse read requests when account context is missing, unauthorized, or ambiguous.
- **FR-005**: System MUST refuse to pick a contract silently when symbol, currency, asset class, or exchange context is ambiguous.
- **FR-006**: System MUST support contract search/resolve, market snapshots, and historical bars for MVP-supported asset classes `stock` and `etf`; unsupported asset classes MUST return a structured refusal.
- **FR-007**: System MUST expose only read-only CLI and MCP capabilities in this feature. Order preview, order submit, order cancel, order modify, order approve, paper-write, live-write, sidecar relay, remote public MCP, and direct broker OAuth2 are out of scope for this MVP.
- **FR-008**: System MUST enforce per-tool local read scopes for health, accounts, portfolio, positions, market data, read-only orders, and audit access using `LocalConfigAuth`.
- **FR-009**: System MUST keep local scope enforcement separate from future OAuth/OIDC. Token expiry, issuer, and audience validation are reserved for the remote MCP OAuth feature, not implemented as fake OAuth in this MVP.
- **FR-010**: System MUST return structured, user-actionable errors with stable code, message, retryability, optional user action, and audit correlation for missing broker session, missing scope, invalid input, missing/ambiguous account, ambiguous contract, unsupported asset class, stale market data, unavailable backend, and unsafe output.
- **FR-011**: System MUST emit audit events for tool calls, denied scopes, completed calls, failed calls, refused calls, broker session checks, and broker session state changes.
- **FR-012**: System MUST redact tokens, cookies, credentials, sensitive headers, local paths, backend session material, and unsafe external text from MCP responses, user-visible errors, logs, snapshots, and audit records.
- **FR-013**: System MUST HMAC-hash account identifiers and sensitive correlation values by default using a local audit secret. Raw SHA-256 MUST NOT be the default for low-entropy identifiers.
- **FR-014**: System MUST support offline validation using broker fixtures or a fake broker backend for account, portfolio, contract, market-data, order-read, error-mapping, keepalive, and audit flows.
- **FR-015**: System MUST provide enough local operator documentation for a user to start the gateway, connect it to a local broker session, inspect read-only data, troubleshoot sessions, and understand why write actions are unavailable.
- **FR-016**: System MUST implement Client Portal Gateway keepalive/tickle and update broker session status when keepalive fails.
- **FR-017**: System MUST validate configuration before serving tools. `verify_tls=false` is allowed only for localhost, `127.0.0.1`, or `::1` Client Portal Gateway URLs.
- **FR-018**: System MUST expose market data status as `live`, `delayed`, `stale`, or `unavailable`, with staleness seconds and warnings when applicable.
- **FR-019**: System MUST use Cargo-discoverable test paths or explicit harnesses so CI cannot silently ignore nested tests.

### Constitutional Requirements *(mandatory for broker, MCP, auth, audit, or order features)*

- **CR-001**: This feature is read-only. It MUST NOT expose order preview, submit, cancel, modify, approve, paper-write, or live-write behavior. Any write-like request MUST fail closed with an audited denial.
- **CR-002**: The touched surfaces are local gateway status, local operator commands, local read-only MCP tools, broker-read abstractions, read-only schemas, configuration, local scopes, error mapping, fake backend, keepalive, and audit records. Broker actions MUST use typed structured inputs, not executable free-form prompts.
- **CR-003**: Required local scopes are `ibkr:health:read`, `ibkr:accounts:read`, `ibkr:portfolio:read`, `ibkr:positions:read`, `ibkr:marketdata:read`, `ibkr:orders:read`, and `ibkr:audit:read`. Missing or disabled local scope MUST deny the tool call and emit a redacted audit event.
- **CR-004**: Minimum audit events are `tool.called`, `tool.denied_scope`, `tool.completed`, `tool.failed`, `tool.refused`, `backend.session.checked`, and `backend.session.changed`. Events MUST include correlation identifiers, tool name, scope, decision, result status, HMAC account hash where applicable, and redaction metadata.
- **CR-005**: Order risk policies are not executed in this read-only feature. However, the feature MUST preserve the future order boundary by refusing order preview, submit, cancel, live trading, prompt-to-trade, and any ambiguous broker action.
- **CR-006**: Remote OAuth/OIDC, direct broker OAuth2, sidecar relay, paper trading, and live trading MUST remain only in `/specs/000-project-roadmap/` and later specs, not in this MVP implementation.

### Key Entities *(include if feature involves data)*

- **Local User**: The person running or authorizing the gateway locally and deciding which account data may be inspected.
- **Auth Context**: Local configuration-derived identity and scope set. This MVP uses `LocalConfigAuth`, not bearer/OAuth tokens.
- **MCP Client**: A compatible local agent client that discovers and calls gateway tools under local scoped authorization.
- **Broker Session Status**: The current usability state of the broker backend, including whether manual user action is required.
- **Broker Account**: An account identifier and safe metadata available to the current user/session.
- **Portfolio Snapshot**: A read-only summary of balances, allocations, cash, margin, and related account state.
- **Position**: A current holding with instrument identity, quantity, currency, and valuation fields.
- **Contract Candidate**: A possible instrument match returned from a contract search, including enough context to avoid silent ambiguity.
- **Market Snapshot**: Read-only bid, ask, last, timestamp, currency, data status, and availability information for a resolved instrument.
- **Historical Bars**: Read-only bar data with duration, bar size, timestamps, data status, and availability warnings.
- **Read-Only Order Record**: Existing broker order, status, or execution data that can be viewed but not changed.
- **Market Data Policy**: Local rule controlling staleness thresholds and delayed-data behavior.
- **Audit Event**: A correlated append-only record of an allowed, denied, refused, failed, or completed operation with redacted sensitive data.
- **Gateway Configuration**: Local settings controlling broker backend mode, read scopes, storage, audit behavior, market-data policy, keepalive, TLS validation, and safety defaults.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A local user can verify broker session status and list available accounts in under 2 minutes after the broker session is already active.
- **SC-002**: 100% of write-like requests in this feature are absent or refused and audited.
- **SC-003**: 100% of exposed MCP tools in this feature are read-only and declare a minimal read scope.
- **SC-004**: 100% of supported read-only fixture scenarios listed in FR-003 and the MCP/CLI contracts return either structured data or a user-actionable structured refusal in offline fixture testing.
- **SC-005**: Audit review can reconstruct which read-only tool was called, which account was involved when applicable, which scope was checked, and whether the call succeeded, failed, or was refused for 100% of significant operations.
- **SC-006**: Secret scanning of user-visible errors, MCP responses, logs, audit fixture output, and contract test snapshots finds zero tokens, cookies, credentials, sensitive headers, local paths, or local backend secrets.
- **SC-007**: Offline performance reporting shows local gateway overhead below 500 ms p95 excluding broker latency, local audit writes below 50 ms p95, and the complete offline fixture suite completing in under 30 seconds.
- **SC-008**: Configuration validation rejects 100% of attempts to enable write tools, remote public MCP, sidecar relay, live trading, or non-localhost TLS bypass in this MVP.

## Assumptions

- The first shippable increment is local, single-user, and read-only.
- Retail/individual users rely on a local Interactive Brokers Client Portal Gateway session that may require manual authentication outside this gateway.
- Remote public MCP, sidecar relay, direct broker OAuth2, order preview, paper submit, cancel, and live trading are later features described in `/specs/000-project-roadmap/`, not part of this spec.
- The gateway may use offline fixtures or a fake broker backend to validate most behavior without requiring a live broker session.
- The user accepts provider-neutral MCP as the integration boundary for agents; direct provider-specific clients are secondary and out of scope for this MVP.
