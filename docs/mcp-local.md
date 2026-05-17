# MCP

The package exposes provider-neutral MCP tooling for local stdio clients and
remote HTTP authorization experiments.

Broker authentication remains separate from MCP authorization. MCP bearer
tokens must never be forwarded to IBKR.

## Local Stdio

```bash
ibkr-agent mcp serve --transport stdio --describe --json
ibkr-agent mcp serve --transport stdio --json
```

Use `--describe` for a smoke check that exits immediately. Omit it when wiring
an MCP client; the command then runs a line-oriented JSON-RPC stdio loop.

The local server lists only tools whose scopes are enabled by the current CLI
config. Every `tools/call` enforces the tool scope before backend access and
writes an audit event for completion, denial, refusal, or failure.

Example client configs live under `examples/mcp-clients/`.

## Remote HTTP

Remote MCP is disabled by default and requires explicit configuration plus the
independent safety flag. See [remote-mcp-oauth.md](remote-mcp-oauth.md).

## Tool Registry

Default broker tools:

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

Live tools are discoverable only when live tool discovery is explicitly enabled
through `broker_tool_schemas_with_live(true)`:

| Tool | Scope |
|------|-------|
| `ibkr_live_order_submit` | `ibkr:orders:live:submit` |
| `ibkr_live_order_cancel` | `ibkr:orders:live:cancel` |

## Forbidden Generic Write Tools

These generic write-like names remain forbidden:

- `ibkr_order_intent_validate`
- `ibkr_order_preview_explain`
- `ibkr_order_submit`
- `ibkr_order_cancel`
- `ibkr_order_modify`
- `ibkr_order_approve`

Use the CLI or SDK preview, paper, or live-gated workflows instead. Direct calls
to forbidden names return `READONLY_WRITE_FORBIDDEN` and are auditable.

## Safety Boundary

Tool outputs must not include broker cookies, tokens, credentials, sensitive
headers, local secret paths, or raw Client Portal Gateway session material.
Scope checks happen before broker access. Missing scope returns
`AUTH_MISSING_SCOPE` and does not call the backend.
