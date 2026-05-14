# Tasks: Remote MCP OAuth/OIDC Front-Door

## Phase 1: HTTP MCP Transport

- [ ] T001 Add HTTP MCP server transport in `crates/ibkr-mcp/src/http_server.rs`.
- [ ] T002 Add request/session id propagation in `crates/ibkr-mcp/src/session.rs`.
- [ ] T003 Add remote server config in `crates/ibkr-config/src/remote_mcp.rs`.
- [ ] T004 Add remote deployment docs in `docs/remote-mcp-oauth.md`.

## Phase 2: OAuth/OIDC Validation

- [ ] T010 Add `crates/ibkr-oauth/Cargo.toml` and `crates/ibkr-oauth/src/lib.rs`.
- [ ] T011 Implement issuer/audience/expiry/signature validation in `crates/ibkr-oauth/src/validator.rs`.
- [ ] T012 Implement JWKS/metadata discovery in `crates/ibkr-oauth/src/jwks.rs`.
- [ ] T013 Implement remote auth context mapping in `crates/ibkr-auth/src/oauth_context.rs`.
- [ ] T014 Implement 401/403 behavior in `crates/ibkr-mcp/src/http_auth.rs`.
- [ ] T015 Implement protected resource metadata config/endpoint in `crates/ibkr-mcp/src/oauth_metadata.rs`.

## Phase 3: Audit and Tests

- [ ] T020 Add auth success/denial audit events in `crates/ibkr-oauth/src/audit.rs`.
- [ ] T021 Add token redaction tests in `tests/replay_remote_token_redaction.rs`.
- [ ] T022 Add remote MCP auth contract tests in `tests/contract_remote_oauth_denials.rs`.
- [ ] T023 Add wrong issuer/audience/expiry/scope tests in `tests/integration_remote_oauth.rs`.
- [ ] T024 Add docs for remote OAuth/OIDC configuration in `docs/remote-mcp-oauth.md`.
