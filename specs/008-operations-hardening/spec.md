# Feature Specification: Operations, Export, Replay, and Hardening

**Feature Branch**: `008-operations-hardening`

**Created**: 2026-05-14

**Status**: Draft

**Input**: Final roadmap phase after read-only, preview/risk, paper submit, remote MCP, sidecar, provider compatibility, and live trading gates.

## Scope Position in Roadmap

Adds operational hardening: audit export, replay, metrics, structured logs, retention, dependency scanning, incident review, and drift detection. It does not add new broker permissions by itself.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Review, Export, and Replay Gateway Behavior (Priority: P1)

As an operator, I want redacted audit export and replayable fixtures, so I can debug incidents and validate changes without exposing secrets or depending on live broker sessions.

**Why this priority**: Once trading and remote access exist, reproducible evidence and regression replay become mandatory.

**Independent Test**: Exported audit data contains no secrets, replay uses fake backends, and replay outcomes match stored expected decisions.

**Acceptance Scenarios**:

1. **Given** recorded redacted audit events, **When** export is requested, **Then** JSONL/SQLite export contains correlation, decisions, scopes, hashes, and no raw secrets.
2. **Given** a replay fixture, **When** replay runs against fake backend, **Then** expected decisions and refusals are reproduced.
3. **Given** metrics are enabled, **When** broker tools are called, **Then** latency, error, refusal, and audit-write metrics are emitted without sensitive labels.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST export redacted audit events as JSONL and/or SQLite backup.
- **FR-002**: System MUST provide replay harnesses for non-secret tool calls and expected decisions.
- **FR-003**: System MUST emit structured logs without secrets.
- **FR-004**: System MUST emit safe metrics for latency, errors, refusals, broker availability, and audit writes.
- **FR-005**: System MUST define audit retention and backup policy.
- **FR-006**: System MUST include dependency/security scanning in CI.
- **FR-007**: System MUST include schema drift detection for MCP tools and contracts.
- **FR-008**: System MUST include incident review documentation.

### Constitutional Requirements

- **CR-001**: Broker actions MUST use typed structured inputs, never executable prompts.
- **CR-002**: Missing scope, disabled config, invalid policy, or failed validation MUST fail closed and be audited.
- **CR-003**: Secrets, tokens, session material, raw headers, credentials, and local paths MUST be redacted from outputs and audit records.
- **CR-004**: This spec MUST NOT weaken any previous read-only, audit, scope, approval, idempotency, sidecar, provider-neutrality, or live-gating guarantee.

## Success Criteria *(mandatory)*

- **SC-001**: 100% of exported audit fixtures pass secret scanning.
- **SC-002**: Replay fixtures reproduce expected structured data or structured refusals.
- **SC-003**: Metrics contain no account IDs, tokens, cookies, credentials, local paths, or raw broker session material.
- **SC-004**: Contract drift detection fails CI when MCP tool schemas change without updated snapshots.

## Assumptions

- All previous specs in the roadmap have been implemented and validated.
- Operations hardening adds observability and reproducibility, not new broker write authority.

