# Feature Specification: Paper Submit and Approval Workflow

**Feature Branch**: `003-paper-submit-approval`

**Created**: 2026-05-14

**Status**: Draft

**Input**: Roadmap phase following the completed local read-only MVP.

## Scope Position in Roadmap

Adds approval records, idempotency, paper-only submit/cancel, and order lifecycle tracking. Live trading remains unavailable.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Submit Paper Orders Only After Approval (Priority: P1)

As a user, I want paper orders submitted only after preview and explicit approval, so I can test trading workflows without live execution.

**Why this priority**: Paper execution validates order lifecycle handling before any live gates exist.

**Independent Test**: Paper submit requires a prior preview hash, approval record, idempotency key, paper account allowlist, and write scope.

**Acceptance Scenarios**:

1. **Given** an approved preview for a paper account, **When** paper submit is called with a new idempotency key, **Then** the gateway submits once and records lifecycle state.
2. **Given** the same idempotency key is replayed, **When** submit is called again, **Then** no duplicate broker action is created.
3. **Given** a live account is used, **When** submit is requested, **Then** the gateway refuses because live trading is out of scope.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST require prior preview and approval before paper submit.
- **FR-002**: System MUST require `ibkr:orders:paper:submit` or `ibkr:orders:paper:cancel` scope for paper write tools.
- **FR-003**: System MUST require idempotency keys for paper submit/cancel.
- **FR-004**: System MUST track order lifecycle states and execution correlation.
- **FR-005**: System MUST reject live accounts and live trading tools.

### Constitutional Requirements

- **CR-001**: Broker actions MUST use typed structured inputs, never executable prompts.
- **CR-002**: Missing scope, disabled config, invalid policy, or failed validation MUST fail closed and be audited.
- **CR-003**: Secrets, tokens, session material, raw headers, credentials, and local paths MUST be redacted from outputs and audit records.
- **CR-004**: This spec MUST NOT weaken any previous read-only, audit, scope, or provider-neutrality guarantees.

## Success Criteria *(mandatory)*

- **SC-001**: 100% of fixture-backed positive scenarios produce structured outputs.
- **SC-002**: 100% of fixture-backed negative scenarios produce structured refusals with stable error codes.
- **SC-003**: All new tools map to explicit scopes and audit events.
- **SC-004**: Secret scanning finds zero raw credentials, cookies, tokens, session material, or local paths.

## Assumptions

- All previous specs in the roadmap have been implemented and validated.
- New behavior is disabled by default unless this spec explicitly enables it.
