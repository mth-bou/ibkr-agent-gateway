# Feature Specification: Local Sidecar Relay for Retail CP Gateway

**Feature Branch**: `005-sidecar-relay`

**Created**: 2026-05-14

**Status**: Draft

**Input**: Roadmap phase following the completed local read-only MVP.

## Scope Position in Roadmap

Adds a local sidecar that bridges a retail user’s local CP Gateway to a remote MCP gateway without automating IBKR login or exposing session material to MCP clients.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bridge Local CP Gateway to Remote MCP Safely (Priority: P1)

As an individual IBKR user, I want a local sidecar to connect my local CP Gateway to a remote gateway, so I can use remote MCP clients while keeping IBKR login local.

**Why this priority**: Retail CP Gateway must run/authenticate locally, while remote MCP needs a controlled bridge.

**Independent Test**: When sidecar heartbeat or session binding fails, remote broker access fails closed and no IBKR session secret is exposed.

**Acceptance Scenarios**:

1. **Given** sidecar is paired and healthy, **When** a remote read request is authorized, **Then** it routes to the local CP Gateway through the sidecar.
2. **Given** sidecar disconnects, **When** a remote broker call is attempted, **Then** the gateway refuses with structured unavailable-sidecar error.
3. **Given** IBKR authentication is missing locally, **When** remote requests arrive, **Then** the gateway reports manual local action required.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support sidecar pairing with a stable sidecar identity.
- **FR-002**: System MUST maintain heartbeat and session binding for remote-to-local relay.
- **FR-003**: System MUST fail closed when sidecar or local CP Gateway session is unavailable.
- **FR-004**: System MUST NOT automate IBKR retail browser login.
- **FR-005**: System MUST NOT expose CP Gateway cookies/session material to remote MCP clients.

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
