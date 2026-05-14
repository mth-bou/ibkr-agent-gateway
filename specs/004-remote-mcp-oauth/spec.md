# Feature Specification: Remote MCP OAuth/OIDC Front-Door

**Feature Branch**: `004-remote-mcp-oauth`

**Created**: 2026-05-14

**Status**: Draft

**Input**: Roadmap phase following the completed local read-only MVP.

## Scope Position in Roadmap

Adds HTTP MCP transport and OAuth/OIDC-protected-resource behavior for remote clients. It authenticates MCP clients to the gateway; it does not replace IBKR backend authentication.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Use Remote MCP With Scoped OAuth/OIDC Authorization (Priority: P1)

As a remote MCP user, I want the gateway to validate bearer tokens and scopes before tool execution, so private broker data is not exposed to unauthorized clients.

**Why this priority**: Remote MCP requires stronger authorization than local stdio config scopes.

**Independent Test**: Missing/invalid tokens return 401, insufficient scopes return 403, valid tokens with sufficient scopes can call permitted tools.

**Acceptance Scenarios**:

1. **Given** no token is provided, **When** a remote MCP request arrives, **Then** the gateway returns 401 with protected-resource metadata behavior.
2. **Given** a valid token with insufficient scope, **When** a protected tool is called, **Then** the gateway returns 403 and audits denial.
3. **Given** a valid token with correct audience, issuer, expiry, and scope, **When** a permitted tool is called, **Then** the gateway processes the request without forwarding the MCP token to IBKR.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST add Streamable HTTP MCP transport.
- **FR-002**: System MUST validate bearer token issuer, audience/resource, expiry, signature, and scopes before tool execution.
- **FR-003**: System MUST return 401 for missing/invalid/expired tokens and 403 for insufficient scopes.
- **FR-004**: System MUST implement protected resource metadata and authorization server discovery configuration.
- **FR-005**: System MUST NOT forward MCP client tokens to IBKR backend calls.
- **FR-006**: System MUST keep remote write tools disabled unless earlier write specs are enabled and remote scopes/policies also allow them.

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
