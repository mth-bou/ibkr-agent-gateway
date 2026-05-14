# Tasks: OpenAI, Anthropic, and MCP Client Compatibility

## Phase 1: Compatibility Harnesses

- [ ] T001 Add `crates/ibkr-provider-compat/Cargo.toml` and `crates/ibkr-provider-compat/src/lib.rs`.
- [ ] T002 Add generic MCP inspector compatibility harness in `crates/ibkr-provider-compat/src/generic_mcp.rs`.
- [ ] T003 Add OpenAI remote MCP compatibility example/test in `crates/ibkr-provider-compat/src/openai.rs`.
- [ ] T004 Add Anthropic MCP connector compatibility example/test in `crates/ibkr-provider-compat/src/anthropic.rs`.
- [ ] T005 Add Cursor/Continue smoke examples in `examples/mcp-clients/`.

## Phase 2: Architecture Guards

- [ ] T010 Add dependency checks forbidding provider SDKs in core crates in `tests/contract_no_provider_sdk_in_core.rs`.
- [ ] T011 Add schema compatibility snapshots in `tests/provider_schema_snapshots.rs`.
- [ ] T012 Add auth denial snapshots in `tests/provider_auth_denial_snapshots.rs`.
- [ ] T013 Add redaction snapshots in `tests/provider_redaction_snapshots.rs`.

## Phase 3: Docs

- [ ] T020 Document provider setup without broker secret leakage in `docs/provider-compatibility.md`.
- [ ] T021 Document provider-side approval UX as optional only in `docs/provider-approval-ux.md`.
