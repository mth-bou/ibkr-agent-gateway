# Feature Specification: OpenAI, Anthropic, and MCP Client Compatibility

**Feature Branch**: `006-provider-compatibility`

**Created**: 2026-05-14

**Status**: Draft

**Input**: Roadmap phase following the completed local read-only MVP.

## Scope Position in Roadmap

Adds compatibility harnesses for OpenAI Responses MCP, Anthropic MCP connector, Cursor, Continue, and local MCP clients. It does not add provider dependencies to broker core.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Validate Provider Compatibility Through MCP (Priority: P1)

As a developer, I want major LLM/MCP clients tested against the gateway, so I can prove the gateway is provider-neutral and usable in real agent workflows.

**Why this priority**: Provider compatibility matters, but provider coupling would corrupt the architecture.

**Independent Test**: Compatibility tests call MCP tools through provider/client-specific harnesses while architecture checks prevent provider SDK imports in broker core crates.

**Acceptance Scenarios**:

1. **Given** an OpenAI MCP harness, **When** read-only tools are listed/called, **Then** schemas and outputs are accepted.
2. **Given** an Anthropic MCP harness, **When** tools are configured with scopes, **Then** allowed/denied behavior matches contracts.
3. **Given** dependency checks run, **When** broker crates are inspected, **Then** no OpenAI/Anthropic SDK dependency is present.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST add provider compatibility tests or examples for OpenAI Responses MCP.
- **FR-002**: System MUST add provider compatibility tests or examples for Anthropic MCP connector.
- **FR-003**: System SHOULD add smoke tests for Cursor, Continue, and generic MCP clients.
- **FR-004**: System MUST keep provider SDKs out of broker, risk, audit, auth, backend, and MCP core crates.
- **FR-005**: Provider-specific approval behavior MUST be treated as client-side complement, not a replacement for gateway gates.

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
