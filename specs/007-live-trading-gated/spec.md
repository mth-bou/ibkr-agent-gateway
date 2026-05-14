# Feature Specification: Live Trading Gated Enablement

**Feature Branch**: `007-live-trading-gated`

**Created**: 2026-05-14

**Status**: Draft

**Input**: Roadmap phase following the completed local read-only MVP.

## Scope Position in Roadmap

Adds live trading only after read-only, preview/risk, paper submit, remote auth, sidecar, and provider compatibility are stable. Live is disabled by default and gated by multiple independent controls.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Enable Live Trading Deliberately and Reversibly (Priority: P1)

As an advanced user, I want live trading enabled only with explicit config, scopes, risk limits, approval, account allowlist, and kill switch, so live access is controlled and reversible.

**Why this priority**: Live trading is the highest-risk phase and must be last.

**Independent Test**: Live write tools are absent by default and become discoverable only when every independent gate is satisfied.

**Acceptance Scenarios**:

1. **Given** live mode is disabled, **When** tools are listed, **Then** live submit/cancel tools are absent.
2. **Given** live mode is enabled but a required gate is missing, **When** live submit is attempted, **Then** the gateway refuses with the specific missing gate.
3. **Given** all gates pass and kill switch is inactive, **When** an approved live order is submitted, **Then** it uses idempotency, audit, risk policy, and lifecycle tracking.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST keep live trading disabled by default.
- **FR-002**: System MUST require explicit live config, live account allowlist, live write scope, risk policy pass, approval record, idempotency key, and inactive kill switch.
- **FR-003**: System MUST support emergency kill switch that immediately disables live submit/cancel tools.
- **FR-004**: System MUST enforce notional, symbol, asset-class, quantity, frequency, and session limits.
- **FR-005**: System MUST retain full audit for live write workflows.
- **FR-006**: System MUST provide paper-to-live migration checklist before live enablement.

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
