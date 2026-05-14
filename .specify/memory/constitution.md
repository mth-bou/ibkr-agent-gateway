<!--
Sync Impact Report
Version change: template -> 1.0.0
Modified principles:
- Template principle 1 -> I. MCP Boundary and Provider Neutrality
- Template principle 2 -> II. Deterministic Finance Domain
- Template principle 3 -> III. Read-Only Default and Controlled Order Lifecycle
- Template principle 4 -> IV. Least-Privilege Auth and Scope Enforcement
- Template principle 5 -> V. Auditability, Observability, and Fail-Closed Security
Added sections:
- Operational Constraints
- Development Workflow
Removed sections:
- None
Templates requiring updates:
- UPDATED .specify/templates/plan-template.md
- UPDATED .specify/templates/spec-template.md
- UPDATED .specify/templates/tasks-template.md
- REVIEWED .specify/extensions/git/commands/*.md; no updates required
- REVIEWED AGENTS.md; no updates required
Follow-up TODOs:
- None
-->

# IBKR Agent Gateway Constitution

## Core Principles

### I. MCP Boundary and Provider Neutrality

The gateway MUST expose broker capabilities through typed MCP tools as the
primary integration boundary. OpenAI, ChatGPT, Claude, Cursor, Continue, and
custom agents are clients of that boundary, not dependencies of the finance
core. The LLM MUST NOT communicate directly with Interactive Brokers, hold IBKR
secrets, or bypass the gateway's auth, scope, audit, risk, and approval layers.
Provider-specific clients MAY exist only as secondary adapters outside the core
IBKR and MCP crates.

Rationale: the project value is a controlled broker gateway, not a bot tied to a
single model provider or prompt runtime.

### II. Deterministic Finance Domain

All account, portfolio, market data, contract, order, approval, and risk logic
MUST be represented with strict Rust domain types and deterministic validation.
Free-form prompts MUST NOT be accepted as executable trading instructions.
There MUST NOT be a `place_order(prompt: String)` or equivalent shortcut. Any
order flow MUST start from structured inputs, resolve to an unambiguous
contract, preserve decimal precision for money and quantities, and return typed
decisions or typed errors.

Rationale: financial safety depends on repeatable code paths that can be tested,
replayed, audited, and refused without model interpretation.

### III. Read-Only Default and Controlled Order Lifecycle

The system MUST be read-only by default. Write tools for preview, submit, or
cancel MUST remain unavailable unless explicitly enabled at runtime and guarded
by dedicated scopes. Order submission MUST follow this lifecycle:
`intent -> validate -> preview -> risk decision -> human approval -> submit`.
Submit MUST require a fresh `OrderPreview`, a matching one-use approval token,
and an idempotency key. Market orders, live trading, derivatives, short selling,
ambiguous contracts, expired previews, missing account context, and missing
currency context MUST fail closed by default.

Rationale: accidental trading, duplicate submission, and account-mode confusion
are the highest-impact failure modes in this product.

### IV. Least-Privilege Auth and Scope Enforcement

The MCP front door MUST validate OAuth/OIDC credentials before protected tool
execution. Every MCP tool MUST declare and enforce its minimal scope; broad
scopes MUST NOT imply narrower trading permissions unless explicitly mapped.
The auth layer for MCP clients MUST remain separate from IBKR backend
authentication. Remote and sidecar topologies MUST use short-lived, auditable,
and device-appropriate credentials, and MUST NOT expose inbound access to a
user's local machine.

Rationale: LLM clients, broker sessions, sidecars, and users are separate trust
boundaries and MUST NOT share implicit authority.

### V. Auditability, Observability, and Fail-Closed Security

Every significant operation MUST emit an audit event: tool calls, scope denials,
risk decisions, previews, approvals, submits, cancels, backend session changes,
and failures. Audit payloads MUST use canonical hashes for sensitive request and
response bodies where appropriate, MUST redact tokens, cookies, credentials, and
sensitive headers, and MUST correlate by request, user, session, approval, and
account identifiers. Runtime code MUST use structured logs, metrics, and typed
error codes. Any ambiguity, missing authorization, stale market data, uncertain
backend session, or unsafe output path MUST refuse the action rather than guess.

Rationale: a broker gateway is only useful when it is explainable after the fact
and predictable under error, hostile input, or model hallucination.

## Operational Constraints

The initial implementation MUST be a Rust Cargo workspace with separated crates
for domain, IBKR Client Portal API access, backend abstraction, risk, audit,
OAuth/OIDC, MCP, CLI, sidecar, and server responsibilities as they are added.
The `ibkr-domain` crate MUST remain free of HTTP, MCP, OAuth, LLM, and storage
concerns. The low-level IBKR client MUST remain non-agentic and MUST NOT know
about LLM providers.

The MVP path MUST prioritize local, read-only Client Portal Gateway support
before remote MCP, order preview, paper submit, sidecar relay, or live trading.
Remote direct IBKR OAuth2 support MAY be added only when the target user segment
and IBKR availability are verified. Live trading MUST require all of: a global
enable flag, a user/account permission, a dedicated scope, stronger approval,
and separate hard limits.

Implementations MUST NOT copy official IBKR code or present this project as an
official IBKR product. Compatibility claims MUST be grounded in public
documentation, broker paper-account testing, local fixtures, or recorded
sessions owned by the project.

## Development Workflow

Every feature plan MUST pass the Constitution Check before Phase 0 research and
again after Phase 1 design. Specs MUST state their read/write posture, affected
scopes, audit events, risk decisions, failure modes, and test strategy.

Tests are mandatory for deterministic domain behavior, risk policies, auth and
scope mapping, audit redaction, backend error mapping, MCP schemas, idempotency,
approval-token reuse, and replay stability. Tests MAY be omitted only for
documentation-only changes or explicitly marked exploratory spikes that cannot
ship. Features touching order submission, cancellation, live account access,
sidecar credentials, OAuth validation, or audit retention MUST include offline
integration tests or contract tests before implementation is considered done.

Each user story MUST be independently demonstrable. The first viable increment
SHOULD be local read-only value unless the feature is explicitly about later
phases. All code MUST keep secrets out of logs, audit payloads, MCP responses,
test snapshots, and error messages.

## Governance

This constitution supersedes conflicting repo guidance, templates, and feature
plans for broker access, MCP exposure, financial safety, auth, testing, audit,
and observability. Amendments require an update to this file, a Sync Impact
Report, and review of the Spec Kit templates plus runtime guidance files.

Versioning follows semantic versioning:

- MAJOR: removes or redefines a principle, weakens a safety gate, or changes
  governance in a backward-incompatible way.
- MINOR: adds a principle, section, or materially expands required guidance.
- PATCH: clarifies wording, fixes typos, or updates non-semantic references.

Compliance review is required during planning, task generation, implementation
review, and before any release that touches broker connectivity, auth,
permissions, order flow, sidecar transport, audit storage, or live trading.
Violations MUST be documented in the plan's Complexity Tracking table with the
safer alternative that was rejected and the reason it was insufficient.

**Version**: 1.0.0 | **Ratified**: 2026-05-14 | **Last Amended**: 2026-05-14
