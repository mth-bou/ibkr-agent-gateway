# Feature Specification: Order Preview and Deterministic Risk Engine

**Feature Branch**: `002-order-preview-risk`

**Created**: 2026-05-14

**Status**: Draft

**Input**: Roadmap phase following the completed local read-only MVP.

## Scope Position in Roadmap

Adds typed `OrderIntent`, contract resolution for preview, deterministic risk policy checks, and `OrderPreview`. It still does not submit or cancel orders.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Prepare an Order Without Submitting It (Priority: P1)

As a user, I want an agent to prepare a typed order preview with risk warnings and normalized broker fields, so I can review trade intent without any execution path.

**Why this priority**: Preview is the first safe step toward trading workflows and must exist before paper submit.

**Independent Test**: A preview request returns `OrderPreview` or `RiskRefusal`; submit/cancel tools remain absent and refused.

**Acceptance Scenarios**:

1. **Given** a valid stock/ETF order intent, **When** preview is requested, **Then** the gateway resolves the contract, validates policy, and returns a non-executable preview.
2. **Given** risk policy rejects the intent, **When** preview is requested, **Then** the gateway returns a structured `ORDER_POLICY_REFUSED` refusal.
3. **Given** submit or cancel is attempted, **When** the request reaches the gateway, **Then** it is refused because this phase is preview-only.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST introduce `OrderIntent` as a typed non-executable proposal.
- **FR-002**: System MUST resolve contracts deterministically before preview.
- **FR-003**: System MUST implement `RiskPolicy`, `RiskWarning`, and `RiskRefusal`.
- **FR-004**: System MUST produce `OrderPreview` with account, contract, side, quantity, order type, price constraints, estimated notional, warnings, refusal status, and audit correlation.
- **FR-005**: System MUST expose preview only behind `ibkr:orders:preview` and explicit config enablement.
- **FR-006**: System MUST keep submit/cancel/approval tools absent or refused.

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
