# Tasks: Remote MCP OAuth/OIDC Front-Door

## Phase 1: HTTP MCP Transport

- [X] T001 Add HTTP MCP server transport in `crates/ibkr-mcp/src/http_server.rs`.
- [X] T002 Add request/session id propagation in `crates/ibkr-mcp/src/session.rs`.
- [X] T003 Add remote server config in `crates/ibkr-config/src/remote_mcp.rs`.
- [X] T004 Add remote deployment docs in `docs/remote-mcp-oauth.md`.

## Phase 2: OAuth/OIDC Validation

- [X] T010 Add `crates/ibkr-oauth/Cargo.toml` and `crates/ibkr-oauth/src/lib.rs`.
- [X] T011 Implement issuer/audience/expiry/signature validation in `crates/ibkr-oauth/src/validator.rs`.
- [X] T012 Implement JWKS/metadata discovery in `crates/ibkr-oauth/src/jwks.rs`.
- [X] T013 Implement remote auth context mapping in `crates/ibkr-auth/src/oauth_context.rs`.
- [X] T014 Implement 401/403 behavior in `crates/ibkr-mcp/src/http_auth.rs`.
- [X] T015 Implement protected resource metadata config/endpoint in `crates/ibkr-mcp/src/oauth_metadata.rs`.

## Phase 3: Audit and Tests

- [X] T020 Add auth success/denial audit events in `crates/ibkr-oauth/src/audit.rs`.
- [X] T021 Add token redaction tests in `tests/replay_remote_token_redaction.rs`.
- [X] T022 Add remote MCP auth contract tests in `tests/contract_remote_oauth_denials.rs`.
- [X] T023 Add wrong issuer/audience/expiry/scope tests in `tests/integration_remote_oauth.rs`.
- [X] T024 Add docs for remote OAuth/OIDC configuration in `docs/remote-mcp-oauth.md`.
