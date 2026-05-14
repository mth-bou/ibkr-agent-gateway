# Local MCP Setup

This document covers the local MCP surface for `specs/001-gateway-mvp-spec`.

The MVP exposes a local stdio MCP entrypoint only. It does not expose a remote
HTTP MCP server, OAuth/OIDC bearer-token validation, sidecar relay, provider
SDK, order preview, order submit, order cancel, order modify, order approve, or
live trading path.

## Serve Locally

```bash
ibkr-agent mcp serve --transport stdio --json
```

Only `stdio` is accepted in this phase. Any other transport is rejected by local
configuration policy.

## Broker Tools

The US3 MCP registry exposes these read-only broker tools:

| Tool | Scope |
|------|-------|
| `ibkr_health` | `ibkr:health:read` |
| `ibkr_backend_status` | `ibkr:health:read` |
| `ibkr_session_requirements` | `ibkr:health:read` |
| `ibkr_accounts_list` | `ibkr:accounts:read` |
| `ibkr_account_summary` | `ibkr:portfolio:read` |
| `ibkr_positions_list` | `ibkr:positions:read` |
| `ibkr_portfolio_snapshot` | `ibkr:portfolio:read` |
| `ibkr_contracts_search` | `ibkr:marketdata:read` |
| `ibkr_contract_resolve` | `ibkr:marketdata:read` |
| `ibkr_market_snapshot` | `ibkr:marketdata:read` |
| `ibkr_historical_bars` | `ibkr:marketdata:read` |
| `ibkr_orders_list` | `ibkr:orders:read` |
| `ibkr_order_status` | `ibkr:orders:read` |
| `ibkr_executions_list` | `ibkr:orders:read` |
| `ibkr_audit_tail` | `ibkr:audit:read` |

`ibkr_audit_tail` is available after US4 and returns only redacted audit
records.

## Forbidden Tool Names

The read-only MVP must not discover these write-like tool names:

- `ibkr_order_intent_validate`
- `ibkr_order_preview`
- `ibkr_order_preview_explain`
- `ibkr_order_submit`
- `ibkr_order_cancel`
- `ibkr_order_modify`
- `ibkr_order_approve`

If a client tries to call one of those names directly, the gateway returns
`READONLY_WRITE_FORBIDDEN` and records a refused MCP tool event.

## Safety Boundary

Tool outputs must not include broker cookies, tokens, credentials, sensitive
headers, local secret paths, or raw Client Portal Gateway session material.
Scope checks happen before broker access. Missing scope returns
`AUTH_MISSING_SCOPE` and does not call the backend.
