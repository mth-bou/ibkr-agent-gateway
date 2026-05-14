# Feature Specification: IBKR Agent Gateway Read-Only MVP

**Feature Branch**: `001-gateway-mvp-spec`

**Created**: 2026-05-14

**Status**: Draft

**Input**: User description: "$speckit-specify lis le plan ibkr-agent-gateway-plan.md pour comprendre le projet"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Verify Broker Session and Accounts (Priority: P1)

As a local user with an Interactive Brokers Client Portal Gateway session, I want
the gateway to tell me whether the broker backend is usable and to list the
accounts I am allowed to inspect, so I can confirm the system is connected
before exposing any data to an agent.

**Why this priority**: Without a clear broker-session and account-read flow, no
portfolio or market-data workflow can be trusted.

**Independent Test**: Can be tested with a connected broker session and with a
missing session. The connected case returns available accounts without secrets;
the missing-session case returns a clear manual action required message.

**Acceptance Scenarios**:

1. **Given** a local broker gateway session is active, **When** the user checks
   gateway status and requests account discovery, **Then** the system reports a
   usable backend and returns only account identifiers and safe account metadata.
2. **Given** no broker gateway session is active, **When** the user checks
   gateway status or requests accounts, **Then** the system refuses broker data
   access and explains the manual session action required.
3. **Given** a backend error occurs, **When** the user requests account
   discovery, **Then** the system returns a typed, non-secret error that can be
   audited and retried or acted on as appropriate.

---

### User Story 2 - Inspect Portfolio and Market Data Read-Only (Priority: P2)

As a local user, I want an agent or CLI session to read portfolio summaries,
positions, contract candidates, market snapshots, historical bars, and recent
orders without write permissions, so I can ask questions about my account
without risking an order being created.

**Why this priority**: Read-only portfolio and market inspection is the first
valuable agent workflow and establishes the safety boundary before any order
preview or execution work.

**Independent Test**: Can be tested using offline broker fixtures and a connected
paper or live account. Each read-only action returns structured data, and no
write action is exposed or accepted.

**Acceptance Scenarios**:

1. **Given** the user has selected an accessible account, **When** positions and
   portfolio summary are requested, **Then** the system returns structured
   balances, allocations, and positions for that account only.
2. **Given** a contract lookup query is ambiguous, **When** the user searches for
   an instrument, **Then** the system returns candidate matches or a clear
   ambiguity refusal rather than selecting one silently.
3. **Given** the user or agent attempts to submit, cancel, or preview an order in
   the read-only MVP, **When** the request reaches the gateway, **Then** the
   system refuses it because write tools are unavailable in this feature.

---

### User Story 3 - Use Read-Only MCP Tools from Local Agents (Priority: P3)

As a local agent user, I want supported MCP clients to discover and call only the
read-only IBKR tools, so I can use agent workflows across compatible clients
without coupling the broker gateway to one provider.

**Why this priority**: MCP is the provider-neutral integration boundary, but it
depends on the broker-read and audit foundations from the first two stories.

**Independent Test**: Can be tested with an MCP-compatible local client that
lists tools, calls read-only tools, and confirms no write tool is present.

**Acceptance Scenarios**:

1. **Given** a local MCP client connects to the gateway, **When** it lists
   available tools, **Then** it sees only health/session, account, portfolio,
   position, contract, market-data, and read-only order-status capabilities.
2. **Given** an MCP client lacks a required read scope, **When** it calls a
   protected read tool, **Then** the system denies the call and records the
   denied scope without leaking tokens or credentials.
3. **Given** an MCP tool returns broker data, **When** the result is sent back to
   the agent context, **Then** the result excludes credentials, cookies, private
   backend headers, local filesystem paths, and other secrets.

---

### User Story 4 - Audit Read-Only Activity (Priority: P4)

As a user reviewing gateway behavior, I want significant tool calls and backend
state changes to be auditable, so I can understand what an agent read, which
account it touched, and why a request was allowed or refused.

**Why this priority**: Audit is required before expanding from read-only access
to order previews, approvals, or paper trading.

**Independent Test**: Can be tested by performing allowed and denied read-only
operations, then reviewing the audit stream for correlated, redacted events.

**Acceptance Scenarios**:

1. **Given** a read-only tool call succeeds, **When** the audit log is reviewed,
   **Then** it includes the tool name, user/session correlation, account
   correlation where applicable, scope used, decision, timestamp, and result
   status.
2. **Given** a request is denied due to missing scope, unavailable backend, or
   ambiguous input, **When** the audit log is reviewed, **Then** it records the
   denial reason without storing secrets.
3. **Given** a sensitive input or output field exists, **When** the audit event
   is written, **Then** the field is redacted or represented by a canonical hash
   instead of raw secret material.

### Edge Cases

- Broker session is absent, expired, or requires manual reauthentication.
- User has multiple accounts and does not specify which account to inspect.
- Requested account is not available to the current user/session.
- Contract search returns multiple plausible instruments for the same symbol.
- Market data is unavailable, delayed, stale, or missing a currency context.
- Historical bars are unavailable for the requested contract, duration, or bar
  size.
- Broker backend returns partial data, rate limits, or a transient error.
- MCP client attempts a write operation even though the feature is read-only.
- External broker-provided text contains prompt-injection style content.
- Error handling or audit logging receives tokens, cookies, headers, or local
  paths that must not be exposed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST expose a local gateway status flow that distinguishes
  usable broker session, unavailable backend, and manual user action required.
- **FR-002**: System MUST list accessible broker accounts with safe metadata
  limited to account identifier, optional display label, account mode, and base
  currency, without exposing credentials, cookies, backend session secrets, raw
  headers, or local secret paths.
- **FR-003**: System MUST allow read-only retrieval of account summary,
  positions, portfolio snapshot, contract candidates, market snapshot,
  historical bars when available, recent orders, order status, and executions
  for an explicitly selected account when the current user/session is allowed to
  inspect it.
- **FR-004**: System MUST refuse read requests when account context is missing,
  unauthorized, or ambiguous.
- **FR-005**: System MUST refuse to pick a contract silently when symbol,
  currency, asset class, or exchange context is ambiguous.
- **FR-006**: System MUST expose only read-only MCP capabilities in this feature.
  Order preview, order submit, order cancel, live trading, sidecar relay, and
  remote public gateway behavior are out of scope for this MVP.
- **FR-007**: System MUST enforce per-tool read scopes for health, accounts,
  portfolio, positions, market data, read-only orders, and audit access.
- **FR-008**: System MUST return structured, user-actionable errors with stable
  code, message, retryability, optional user action, and audit correlation for
  missing broker session, missing scope, invalid input, ambiguous contract,
  stale market data, unavailable backend, and unsafe output.
- **FR-009**: System MUST emit audit events for tool calls, denied scopes,
  completed calls, failed calls, and broker session state changes.
- **FR-010**: System MUST redact tokens, cookies, credentials, sensitive
  headers, and local paths from MCP responses, user-visible errors, logs, and
  audit records.
- **FR-011**: System MUST support offline validation using broker fixtures or a
  fake broker backend for account, portfolio, contract, market-data, order-read,
  error-mapping, and audit flows.
- **FR-012**: System MUST provide enough local operator documentation for a user
  to start the gateway, connect it to a local broker session, inspect read-only
  data, and understand why write actions are unavailable.

### Constitutional Requirements *(mandatory for broker, MCP, auth, audit, or order features)*

- **CR-001**: This feature is read-only. It MUST NOT expose order preview,
  submit, cancel, paper-write, or live-write behavior. Any write-like request
  MUST fail closed with an audited denial.
- **CR-002**: The touched surfaces are local gateway status, local operator
  commands, read-only MCP tools, broker-read abstractions, read-only schemas,
  configuration, error mapping, and audit records. Broker actions MUST use typed
  structured inputs, not executable free-form prompts.
- **CR-003**: Required scopes are `ibkr:health:read`, `ibkr:accounts:read`,
  `ibkr:portfolio:read`, `ibkr:positions:read`, `ibkr:marketdata:read`,
  `ibkr:orders:read`, and `ibkr:audit:read`. Missing, expired, wrong-audience,
  or insufficient authorization MUST deny the tool call and emit a redacted audit
  event.
- **CR-004**: Minimum audit events are `tool.called`, `tool.denied_scope`,
  `tool.completed`, `tool.failed`, and `backend.session.changed`. Events MUST
  include correlation identifiers, tool name, scope, decision, result status,
  and redaction metadata where applicable.
- **CR-005**: Order risk policies are not executed in this read-only feature.
  However, the feature MUST preserve the future order boundary by refusing order
  preview, submit, cancel, live trading, prompt-to-trade, and any ambiguous
  broker action.

### Key Entities *(include if feature involves data)*

- **Local User**: The person running or authorizing the gateway locally and
  deciding which account data may be inspected.
- **MCP Client**: A compatible local agent client that discovers and calls
  gateway tools under scoped authorization.
- **Broker Session Status**: The current usability state of the broker backend,
  including whether manual user action is required.
- **Broker Account**: An account identifier and safe metadata available to the
  current user/session.
- **Portfolio Snapshot**: A read-only summary of balances, allocations, cash,
  margin, and related account state.
- **Position**: A current holding with instrument identity, quantity, currency,
  and valuation fields.
- **Contract Candidate**: A possible instrument match returned from a contract
  search, including enough context to avoid silent ambiguity.
- **Market Snapshot**: Read-only bid, ask, last, timestamp, currency, and
  availability information for a resolved instrument.
- **Read-Only Order Record**: Existing broker order, status, or execution data
  that can be viewed but not changed.
- **Audit Event**: A correlated record of an allowed or denied operation with
  redacted sensitive data.
- **Gateway Configuration**: Local settings controlling broker backend mode,
  read scopes, storage, audit behavior, and safety defaults.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A local user can verify broker session status and list available
  accounts in under 2 minutes after the broker session is already active.
- **SC-002**: 100% of write-like requests in this feature are refused and audited.
- **SC-003**: 100% of exposed MCP tools in this feature are read-only and declare
  a minimal read scope.
- **SC-004**: At least 95% of supported read-only calls listed in FR-003 and
  the MCP/CLI contracts return either structured data or a user-actionable
  structured refusal in offline fixture testing.
- **SC-005**: Audit review can reconstruct which read-only tool was called, which
  account was involved when applicable, which scope was checked, and whether the
  call succeeded or failed for 100% of significant operations.
- **SC-006**: Secret scanning of user-visible errors, MCP responses, logs,
  audit fixture output, and contract test snapshots finds zero tokens, cookies,
  credentials, sensitive headers, local paths, or local backend secrets.
- **SC-007**: Offline performance reporting shows local gateway overhead below
  500 ms p95 excluding broker latency, local audit writes below 50 ms p95, and
  the complete offline fixture suite completing in under 30 seconds.

## Assumptions

- The first shippable increment is local, single-user, and read-only.
- Retail/individual users rely on a local Interactive Brokers Client Portal
  Gateway session that may require manual authentication outside this gateway.
- Remote public MCP, sidecar relay, direct broker OAuth2, order preview, paper
  submit, cancel, and live trading are later features, not part of this spec.
- The gateway may use offline fixtures or a fake broker backend to validate most
  behavior without requiring a live broker session.
- The user accepts provider-neutral MCP as the integration boundary for agents;
  direct provider-specific clients are secondary and out of scope for this MVP.
