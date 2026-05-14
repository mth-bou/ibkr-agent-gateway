# Quickstart: IBKR Agent Gateway Complete Roadmap

This quickstart is the recommended build order for starting from zero without losing the full product direction.

## 1. Start With the Local Read-Only MVP

Implement `/specs/001-gateway-mvp-spec/` first.

Expected result:

- local Client Portal Gateway read backend;
- fake backend fixtures;
- CLI read-only commands;
- local MCP stdio read-only tools;
- local scopes;
- audit recorder and audit tail;
- no order preview, submit, cancel, sidecar, remote MCP, or live trading.

## 2. Add Order Preview and Risk

Create `/specs/002-order-preview-risk/`.

Expected result:

- `OrderIntent`;
- deterministic `RiskPolicy`;
- `ValidatedOrder`;
- `OrderPreview`;
- no submit/cancel.

## 3. Add Paper Submit and Approval

Create `/specs/003-paper-submit-approval/`.

Expected result:

- explicit approval;
- idempotency keys;
- paper account allowlist;
- submit/cancel paper only;
- order lifecycle tracking;
- no live writes.

## 4. Add Remote MCP OAuth/OIDC

Create `/specs/004-remote-mcp-oauth/`.

Expected result:

- HTTP MCP transport;
- OAuth/OIDC JWT validation;
- issuer/audience/scope checks;
- auth metadata;
- no broker secrets in client-visible surfaces.

## 5. Add Sidecar Relay

Create `/specs/005-sidecar-relay/`.

Expected result:

- local sidecar process;
- remote relay session;
- heartbeat/disconnect handling;
- local Client Portal Gateway remains local;
- remote MCP clients still see only tools, never local broker secrets.

## 6. Add Provider Compatibility

Create `/specs/006-provider-compatibility/`.

Expected result:

- OpenAI remote MCP compatibility tests;
- Anthropic remote MCP compatibility tests;
- Cursor/Continue/local MCP inspector tests;
- no provider-specific broker logic.

## 7. Add Live Trading Gates Last

Create `/specs/007-live-trading-gated/`.

Expected result:

- live disabled by default;
- explicit live account allowlist;
- risk limits;
- confirmation policy;
- kill switch;
- audit retention;
- smoke tests gated by explicit environment flags.
