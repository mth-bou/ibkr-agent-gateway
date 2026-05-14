# Feature Specification: IBKR Agent Gateway Complete Roadmap

**Feature Branch**: `000-project-roadmap`

**Created**: 2026-05-14

**Status**: Draft

**Input**: Product vision: Rust-first MCP/OAuth gateway for Interactive Brokers, provider-neutral, auditable, scoped, deterministic, with read-only first delivery, future order preview/risk, paper approvals, remote OAuth MCP, local sidecar relay, provider compatibility, and gated live trading.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read Broker Data Safely Through a Local Gateway (Priority: P1)

As a local user, I want to connect the gateway to a manually authenticated Interactive Brokers Client Portal Gateway session and expose only read-only data to CLI/MCP clients, so I can inspect my portfolio through agents without any trading capability.

**Why this priority**: This establishes the broker adapter, typed domain model, local MCP boundary, scopes, audit, fixtures, and read-only fail-closed posture that every later feature depends on.

**Independent Test**: A local fake backend and a local Client Portal Gateway session can list status, accounts, positions, portfolio snapshots, supported market data, read-only orders/executions, MCP tools, and audit entries without exposing or accepting any write action.

---

### User Story 2 - Prepare Orders Without Sending Them (Priority: P2)

As a user, I want an agent to produce a typed `OrderIntent` and receive a deterministic risk-checked order preview without broker submission, so I can evaluate a potential trade while keeping execution impossible.

**Why this priority**: Order preview is the first write-adjacent capability. It must be separated from submit/cancel and must prove that the LLM proposes while deterministic Rust code validates.

**Independent Test**: Valid and invalid order intents produce `ValidatedOrder`, `OrderPreview`, risk warnings, or typed refusals. No submit/cancel endpoint, command, or MCP tool is present in this feature.

---

### User Story 3 - Submit and Cancel Paper Orders With Explicit Approval (Priority: P3)

As a user operating a paper account, I want order submission and cancellation to require explicit approval, idempotency, audit correlation, and lifecycle tracking, so execution is testable without live capital risk.

**Why this priority**: Paper trading validates the full lifecycle before live trading. It adds the approval boundary, order state machine, and broker write adapter while still banning live writes.

**Independent Test**: Approved paper orders can be submitted once, duplicate idempotency keys do not resubmit, cancellations are audited, and lifecycle events are streamed/read without exposing live trading.

---

### User Story 4 - Protect Remote MCP With OAuth/OIDC Scopes (Priority: P4)

As a remote user or agent client, I want the gateway to expose MCP over HTTP only behind OAuth/OIDC authorization and per-tool scopes, so OpenAI, Anthropic, Cursor, Continue, or other MCP clients can call tools without receiving broker secrets.

**Why this priority**: Remote usage requires a proper front door before the gateway can safely leave a local-only topology.

**Independent Test**: Missing, expired, wrong-audience, wrong-issuer, and insufficient-scope tokens are denied and audited. Authorized tokens can access only the permitted MCP tools.

---

### User Story 5 - Bridge Remote MCP to a Local Client Portal Gateway Sidecar (Priority: P5)

As a retail/individual IBKR user whose broker session is local and manually authenticated, I want a local sidecar to connect my Client Portal Gateway to a remote MCP server without exposing broker cookies or credentials, so remote agents can use my local broker session safely.

**Why this priority**: Retail Client Portal Gateway auth is local/manual. A remote server cannot magically authenticate it. The sidecar/relay topology solves that without pretending broker OAuth is universal.

**Independent Test**: A sidecar establishes a mutually authenticated relay session, forwards only allowed broker operations, handles heartbeat/disconnects, and prevents remote clients from seeing local broker session material.

---

### User Story 6 - Validate Provider Compatibility Without Provider Lock-In (Priority: P6)

As a developer, I want the same MCP tool server to work with OpenAI, Anthropic, Cursor, Continue, and local MCP inspectors without changing broker logic, so the project remains provider-neutral.

**Why this priority**: The project value is a broker gateway, not a hard-coded OpenAI or Anthropic wrapper.

**Independent Test**: Provider compatibility tests prove that the MCP tool schemas, auth behavior, errors, and redactions work across target clients while no provider-specific code exists in `ibkr-domain`, `ibkr-backend`, `ibkr-risk`, or order execution logic.

---

### User Story 7 - Enable Live Trading Only Behind Strict Gates (Priority: P7)

As an advanced user, I want live trading to remain disabled unless explicitly configured with account allowlists, limits, confirmations, kill switches, and audit retention, so production use is possible without accidental live writes.

**Why this priority**: Live execution is the highest-risk capability and must be last. It should build on proven read-only, preview, paper, OAuth, sidecar, and audit layers.

**Independent Test**: Live trading is impossible by default. Enabling it requires explicit config, account allowlist, risk policy, approval policy, kill-switch state, idempotency, and complete audit logging.

### Edge Cases

- Retail/individual IBKR users can authenticate only through a local Client Portal Gateway session.
- Direct IBKR OAuth2 may be available only for specific account types, organizations, advisors, brokers, or future product availability.
- MCP clients may attempt tool calls without tokens, with stale tokens, with insufficient scopes, or with wrong audience/issuer.
- Agent-generated order requests may be ambiguous, unsupported, oversized, duplicate, stale, or inconsistent with account state.
- Broker APIs may return stale market data, delayed data, partial data, rate limits, session expiry, backend failures, or misleading text fields.
- Remote sidecar relay may disconnect, reconnect, duplicate requests, or receive calls while the local broker session has expired.
- Live and paper accounts may share similar identifiers; account mode must be explicit.
- Prompt-injection style content may appear in broker-provided descriptions, news, issuer names, or external text.
- Audit storage may become unavailable; trading actions must not proceed if audit requirements cannot be met for write-capable phases.
- Provider-specific MCP clients may render tool schemas, auth metadata, errors, or streaming events differently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project MUST be Rust-first with a Cargo workspace that keeps domain, broker adapters, MCP transport, OAuth/auth, risk, orders, audit, sidecar, CLI, and provider compatibility concerns separated.
- **FR-002**: The broker-facing core MUST expose deterministic typed operations, never executable free-form prompts.
- **FR-003**: The first shippable increment MUST be local, single-user, read-only, and compatible with a local Interactive Brokers Client Portal Gateway session.
- **FR-004**: The gateway MUST expose provider-neutral MCP tools as the primary agent integration boundary.
- **FR-005**: The gateway MUST also expose a CLI/operator surface for local debugging, fixtures, audit review, and manual validation.
- **FR-006**: Local read-only scope enforcement MUST exist before remote OAuth/OIDC is implemented.
- **FR-007**: Remote MCP MUST be introduced only behind OAuth/OIDC authorization, audience validation, issuer validation, per-tool scopes, and redacted audit events.
- **FR-008**: IBKR backend authentication MUST remain separate from MCP client authorization. MCP clients MUST NOT receive IBKR cookies, credentials, tokens, local secret paths, or raw broker headers.
- **FR-009**: The project MUST support at least two future broker auth/backend modes: local Client Portal Gateway and direct broker OAuth2 where available. Optional TWS/IB Gateway support may be added later behind the same backend abstraction.
- **FR-010**: The project MUST support a fake backend and offline fixtures for deterministic tests before live broker validation.
- **FR-011**: The audit layer MUST be append-only, redacted, correlated, and available before MCP exposure and before any write-capable feature.
- **FR-012**: Account identifiers and sensitive correlation values MUST default to HMAC-SHA256 using a local or deployment secret, not raw SHA-256.
- **FR-013**: Order capabilities MUST progress through separate specs: preview-only, paper submit/cancel with approval, then live trading with additional gates.
- **FR-014**: The write path MUST separate `OrderIntent`, deterministic validation, risk policy, preview, approval, submission, order status, execution events, and cancellation.
- **FR-015**: Order submission MUST use idempotency keys and fail closed on duplicate/unknown state.
- **FR-016**: Live trading MUST be impossible by default and require explicit live account allowlisting, risk limits, approval policy, kill switch, and audit availability.
- **FR-017**: Sidecar relay MUST be introduced only after remote OAuth/OIDC MCP exists and MUST not expose local broker secrets to remote MCP clients.
- **FR-018**: Provider-specific compatibility work MUST test MCP behavior without coupling core broker logic to OpenAI, Anthropic, or any single provider SDK.
- **FR-019**: All external text that enters agent-visible output MUST be treated as data, sanitized/redacted when needed, and never allowed to alter gateway policy.
- **FR-020**: The project MUST document local, remote, sidecar, paper, and live topologies separately.
- **FR-021**: Every feature spec MUST include contracts, data model, tests, failure modes, audit behavior, and forbidden capabilities.
- **FR-022**: Every feature that touches broker access, MCP, auth, audit, risk, or orders MUST define measurable success criteria.

### Constitutional Requirements *(mandatory for broker, MCP, auth, audit, risk, or order features)*

- **CR-001**: The finance domain is deterministic. LLM output may propose structured intents; deterministic Rust code validates, refuses, previews, approves, submits, and audits.
- **CR-002**: MCP is the provider-neutral boundary. Provider adapters and compatibility tests must not contaminate broker, domain, risk, or order execution crates.
- **CR-003**: IBKR secrets stay server/sidecar-side. They are never returned to MCP clients, model contexts, logs, user-visible errors, or audit records.
- **CR-004**: Missing account context, unsupported asset class, ambiguous contract, stale market data, missing scope, unavailable audit storage for writes, and unavailable broker session fail closed.
- **CR-005**: Write capabilities are unavailable until their specific feature spec enables them. No feature may silently smuggle order preview, submit, cancel, or live trading into an earlier phase.
- **CR-006**: Audit is required for allowed calls, denied calls, failures, broker session checks/changes, risk decisions, approvals, submissions, cancellations, and kill-switch changes.
- **CR-007**: Remote MCP requires OAuth/OIDC front-door authorization. Local Client Portal Gateway authentication is not a substitute for MCP client authorization.
- **CR-008**: Paper and live accounts must be explicitly separated in data models, config, audit events, and command/tool behavior.
- **CR-009**: Live trading requires a kill switch, account allowlist, notional/quantity limits, approval policy, idempotency, and audit availability.

### Key Entities *(include if feature involves data)*

- **Gateway Instance**: A local, remote, or sidecar-capable gateway process exposing CLI/MCP surfaces.
- **Broker Backend**: A typed adapter for Client Portal Gateway, future IBKR OAuth2 Web API, fake fixtures, and optional future TWS/IB Gateway.
- **MCP Client**: A provider-neutral client invoking gateway tools locally or remotely.
- **Auth Context**: LocalConfig, OAuth/OIDC token, or sidecar session identity with scopes and audience.
- **Broker Account**: A visible IBKR account with mode, base currency, and safe metadata.
- **Contract Candidate / Resolved Contract**: Instrument identity used for market data and orders.
- **Market Data Record**: Snapshot or bar data with timestamp, data status, staleness, and warnings.
- **Order Intent**: LLM/user-proposed structured order request, not executable by itself.
- **Risk Policy**: Deterministic constraints such as account mode, notional, quantity, asset class, concentration, cooldown, and live/paper gates.
- **Validated Order**: A risk-checked broker-order candidate eligible for preview.
- **Order Preview**: Broker or local estimate/warnings for a validated order without submission.
- **Approval Request**: Human or policy approval record binding a preview to a later submit attempt.
- **Order Receipt / Order Status**: Broker response and lifecycle state for submitted paper/live orders.
- **Sidecar Session**: Mutually authenticated relay connection from local broker environment to remote MCP gateway.
- **Audit Event**: Append-only redacted event with correlation, decision, scope, HMAC identifiers, and result status.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The local read-only MVP can be validated offline with 100% structured data or structured refusals for all supported fixture scenarios.
- **SC-002**: 100% of write-like calls are absent or refused and audited until a dedicated write-capable feature spec enables them.
- **SC-003**: 100% of MCP tools declare exactly one minimal required scope or a documented scope set.
- **SC-004**: 100% of remote MCP requests without valid authorization are denied and audited once remote MCP is implemented.
- **SC-005**: Order preview can be enabled without adding submit/cancel capabilities in the same spec.
- **SC-006**: Paper submit/cancel can be enabled without live trading in the same spec.
- **SC-007**: Live trading is disabled by default and cannot be enabled without explicit config, account allowlist, risk policy, approval policy, audit availability, and kill-switch state.
- **SC-008**: Provider compatibility tests demonstrate OpenAI, Anthropic, Cursor/Continue, and local MCP inspector behavior without provider-specific broker logic.
- **SC-009**: Secret scanning finds zero broker secrets, tokens, cookies, local paths, or sensitive headers in MCP responses, user-visible errors, logs, audit snapshots, fixtures, and provider compatibility snapshots.

## Assumptions

- The project starts from zero and intentionally avoids copying IBKR official client code.
- The first target user is a local retail/individual IBKR user using Client Portal Gateway.
- MCP is the default agent integration strategy; direct OpenAI/Anthropic SDK usage is optional compatibility work, not core architecture.
- Direct broker OAuth2 support is future/conditional and must be implemented behind the same backend abstraction.
- Sidecar relay is required for a realistic remote MCP product that still supports local/manual Client Portal Gateway authentication.
- Live trading is a late feature and must not be inferred from read-only, preview, or paper specs.
