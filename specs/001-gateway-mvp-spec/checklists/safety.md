# Safety Requirements Checklist: IBKR Agent Gateway Read-Only MVP

**Purpose**: Validate clarity, completeness, and consistency of requirements for read-only broker access, MCP scope boundaries, auditability, redaction, keepalive, and secret handling before implementation.
**Created**: 2026-05-14
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 Are all read-only broker data surfaces explicitly listed, including session, accounts, portfolio, positions, contracts, market snapshots, historical bars, orders, executions, and audit tail? [Completeness, Spec §FR-003, Spec §CR-002]
  - Evidence: `spec.md` lists all broker read surfaces; `contracts/mcp-tools.md` separates broker read tools from US4 audit tail.
- [x] CHK002 Are forbidden write capabilities explicitly documented across spec, contracts, and tasks, including preview, submit, cancel, modify, approve, paper-write, live-write, sidecar relay, direct broker OAuth2, and remote public MCP? [Completeness, Spec §FR-007, Spec §CR-001]
  - Evidence: FR-007/CR-001/CR-006 exclude these capabilities; CLI/MCP contracts and tasks include forbidden write refusal checks.
- [x] CHK003 Are required local read scopes enumerated and mapped to protected surfaces? [Completeness, Spec §FR-008, Spec §CR-003]
  - Evidence: CR-003 enumerates read scopes; `contracts/mcp-tools.md` maps each tool to a scope; tasks cover local scope modeling and enforcement.
- [x] CHK004 Is local scope enforcement separated from future OAuth/OIDC? [Completeness, Spec §FR-009]
  - Evidence: FR-009 and `research.md` explicitly reserve token expiry, audience, and issuer validation for a later remote MCP feature.
- [x] CHK005 Are audit event requirements documented for allowed, denied, refused, failed, completed, checked, and changed operations? [Completeness, Spec §FR-011, Spec §CR-004]
  - Evidence: `contracts/audit-events.md` lists required events and tasks implement record-only audit before US1 plus tail in US4.
- [x] CHK006 Are secret redaction requirements documented for MCP responses, user-visible errors, logs, audit records, fixture outputs, and snapshots? [Completeness, Spec §FR-012, Spec §SC-006]
  - Evidence: FR-012/SC-006 define redaction surfaces; tasks include secret scanning.
- [x] CHK007 Is HMAC account hashing required by default? [Completeness, Spec §FR-013]
  - Evidence: FR-013, `data-model.md`, `contracts/config.md`, and `contracts/audit-events.md` require HMAC-SHA256 by default.
- [x] CHK008 Is keepalive/tickle behavior covered? [Completeness, Spec §FR-016]
  - Evidence: FR-016, US1 acceptance scenario 4, `data-model.md`, and tasks T042/T045/T049/T052/T092 cover keepalive.

## Requirement Clarity

- [x] CHK009 Is "read-only" defined with enough specificity to exclude all order preview, approval, submit, cancel, modification, and live trading paths? [Clarity, Spec §FR-007, Spec §CR-001]
- [x] CHK010 Is "safe metadata" clear enough to distinguish allowed account metadata from credentials, cookies, and backend session material? [Clarity, Spec §FR-002]
- [x] CHK011 Is the meaning of structured errors defined enough to guide broker session, scope, ambiguity, stale data, unsupported asset class, invalid config, and unsafe output cases? [Clarity, Spec §FR-010]
- [x] CHK012 Are ambiguity refusal requirements specific for missing account context and ambiguous contract resolution? [Clarity, Spec §FR-004, Spec §FR-005]
- [x] CHK013 Are asset-class limits explicit enough to prevent accidental options/futures/derivatives support in the MVP? [Clarity, Spec §FR-006]
- [x] CHK014 Is market-data staleness/delay handling explicit? [Clarity, Spec §FR-018]
- [x] CHK015 Is TLS verification bypass constrained to localhost? [Clarity, Spec §FR-017]

## Requirement Consistency

- [x] CHK016 Do the spec, MCP contract, CLI contract, and tasks consistently include historical bars as a read-only market-data surface? [Consistency]
- [x] CHK017 Do the spec and plan consistently separate MCP client authorization from IBKR Client Portal Gateway session authentication? [Consistency]
- [x] CHK018 Do the plan, data model, config contract, and tasks consistently assign runtime configuration to `ibkr-config` rather than `ibkr-domain`? [Consistency]
- [x] CHK019 Are audit event names consistent between spec, audit contract, and task coverage? [Consistency]
- [x] CHK020 Is `ibkr_audit_tail` consistently assigned to US4 rather than US3? [Consistency]
- [x] CHK021 Are test paths consistently Cargo-discoverable or explicitly harnessed? [Consistency]
- [x] CHK022 Are CPAPI endpoint mappings and stable error codes now documented? [Consistency]

## Acceptance Criteria Quality

- [x] CHK023 Can each user story acceptance criterion be traced to at least one functional requirement and at least one planned task group? [Traceability]
- [x] CHK024 Are success criteria objective and measurable? [Measurability]
- [x] CHK025 Is the "100% write-like requests refused and audited" requirement supported by precise definitions of write-like requests? [Measurability]
- [x] CHK026 Is the offline fixture success criterion 100% for deterministic supported scenarios? [Measurability]
- [x] CHK027 Is the secret scanning success criterion scoped to all artifacts that can contain sensitive material? [Measurability]

## Scenario Coverage

- [x] CHK028 Primary scenarios cover connected broker session, account discovery, read-only inspection, local MCP discovery/calls, and audit review.
- [x] CHK029 Exception scenarios cover missing/expired session, backend error, keepalive failure, missing scope, unauthorized account, ambiguous contract, unsupported asset class, stale market data, unsafe output, and invalid config.
- [x] CHK030 Recovery/manual-action requirements are defined for session absence, expiration, and broker reauthentication needs.
- [x] CHK031 Requirements cover partial broker data, rate limits, transient backend failures, and prompt-injection style external text.

## Non-Functional Requirements

- [x] CHK032 Security requirements are traceable to tokens, cookies, credentials, sensitive headers, local paths, and backend session material.
- [x] CHK033 Auditability requirements cover correlation identifiers, HMAC account hashing, decision, result status, scope, and redaction metadata.
- [x] CHK034 Local-only deployment boundaries exclude public remote MCP and sidecar behavior from this feature.
- [x] CHK035 Performance requirements have measurable thresholds.
- [x] CHK036 Offline validation covers fake backend fixtures, replay, contract tests, keepalive, and secret scanning.

## Dependencies & Assumptions

- [x] CHK037 Assumptions about local single-user operation and manual Client Portal Gateway authentication are explicit.
- [x] CHK038 Later-phase dependencies such as direct IBKR OAuth2, sidecar relay, paper submit, provider compatibility, and live trading are excluded without weakening future extensibility.
- [x] CHK039 The boundary between local scopes and future OAuth/OIDC front-door authentication is clear.
- [x] CHK040 The complete roadmap exists and prevents this MVP from being mistaken for the full product.

## Notes

- Checklist status: 40/40 complete.
- The completed items document requirement-level evidence only. They do not claim implementation is done; implementation remains governed by `tasks.md`.
