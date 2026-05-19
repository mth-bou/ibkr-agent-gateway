# MCP

The package exposes provider-neutral MCP tooling for local stdio clients and
remote HTTP clients.

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

Use `--describe` for a config smoke check, or omit it to bind the HTTP listener
and serve JSON-RPC requests on `POST /mcp`:

```bash
ibkr-agent mcp serve --transport http --describe --enable-remote-mcp --json
ibkr-agent --config config/remote.example.yaml mcp serve --transport http --enable-remote-mcp --bind 127.0.0.1:8080
```

The HTTP transport validates OAuth/OIDC bearer tokens, serves protected-resource
metadata, filters tool discovery by granted scopes, and routes authorized
`tools/call` requests through the same handlers as stdio.

## Tool Registry

Read tools:

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

Preview and paper tools are discoverable when their scopes are enabled:

| Tool | Scope |
|------|-------|
| `ibkr_order_preview` | `ibkr:orders:preview` |
| `ibkr_paper_order_submit` | `ibkr:orders:paper:submit` |
| `ibkr_paper_order_cancel` | `ibkr:orders:paper:cancel` |

Live tools are discoverable when live scopes are enabled:

| Tool | Scope |
|------|-------|
| `ibkr_live_order_submit` | `ibkr:orders:live:submit` |
| `ibkr_live_order_cancel` | `ibkr:orders:live:cancel` |

Live submit arguments are `account_id`, `approval_id`, `preview_id`, and
`idempotency_key`. The handler loads approval, preview, live policy, writer,
market snapshot, and audit state from the server runtime; these values are not
trusted from the MCP payload. Successful submits are added to the live
reconciliation backlog. Cancel results preserve the broker status and only
terminal states are removed from pending reconciliation.

Planned maturity tools from `specs/009-mcp-tool-maturity/` are not part of the
current registry until their implementation phase lands. The first planned
additions are consultative and safety read tools: PnL, order history, account
metadata, kill switch status, live limits status, MCP audit export, and explicit
session renewal. Later phases keep write-capable additions explicit, such as
`ibkr_paper_order_modify` and `ibkr_live_order_modify`, while the generic
`ibkr_order_modify` name remains forbidden.

## Forbidden Generic Write Tools

These generic write-like names remain forbidden:

- `ibkr_order_intent_validate`
- `ibkr_order_preview_explain`
- `ibkr_order_submit`
- `ibkr_order_cancel`
- `ibkr_order_modify`
- `ibkr_order_approve`

Use the explicit preview, paper, or live-gated tools instead. Direct calls to
forbidden names return `READONLY_WRITE_FORBIDDEN` and are auditable.

## Safety Boundary

Tool outputs must not include broker cookies, tokens, credentials, sensitive
headers, local secret paths, or raw Client Portal Gateway session material.
Scope checks happen before broker access. Missing scope returns
`AUTH_MISSING_SCOPE` and does not call the backend.
