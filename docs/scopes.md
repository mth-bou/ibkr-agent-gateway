# Scopes

Scopes are explicit gateway permissions. Local scopes are loaded from
configuration or test harnesses; remote MCP scopes are granted only after OAuth
token validation and intersection with `remote_mcp.allowed_scopes`.

IBKR broker authentication is separate from gateway scopes.

## Read Scopes

| Scope | Purpose |
|-------|---------|
| `ibkr:health:read` | health, backend status, session requirements |
| `ibkr:accounts:read` | account discovery |
| `ibkr:portfolio:read` | account summary and portfolio snapshot |
| `ibkr:positions:read` | positions |
| `ibkr:marketdata:read` | contract search/resolve, snapshots, bars |
| `ibkr:orders:read` | read-only orders and executions |
| `ibkr:audit:read` | redacted audit tail |

## Preview, Paper, and Live Scopes

| Scope | Purpose |
|-------|---------|
| `ibkr:orders:preview` | non-executable order preview |
| `ibkr:risk:read` | risk policy/risk result inspection |
| `ibkr:orders:paper:submit` | paper submit lifecycle |
| `ibkr:orders:paper:cancel` | paper cancel lifecycle |
| `ibkr:orders:live:submit` | live submit through the live order writer |
| `ibkr:orders:live:cancel` | live cancel through the live order writer |

Preview, paper, and live scopes do not bypass feature flags, approvals,
idempotency, risk limits, kill switch, audit availability, or migration
checklists.

## Planned Maturity Scopes

`specs/009-mcp-tool-maturity/` reserves the next scope families before they are
implemented:

| Scope | Purpose |
|-------|---------|
| `ibkr:audit:export` | redacted MCP audit export, stronger than audit tail |
| `ibkr:orders:paper:modify` | paper order modification lifecycle |
| `ibkr:orders:live:modify` | live-gated order modification lifecycle |
| `ibkr:options:read` | options chain and greeks |
| `ibkr:marketdata:depth:read` | bounded Level II/depth reads |
| `ibkr:scanner:read` | allowlisted market scanners |
| `ibkr:news:read` | bounded broker news metadata and articles |
| `ibkr:fundamentals:read` | bounded fundamentals reports |
| `ibkr:calendar:read` | holidays and market session status |
| `ibkr:currency:read` | read-only FX rates |
| `ibkr:transfers:read` | redacted transfer history |
| `ibkr:approvals:create` | MCP-created gateway approval records |

## MCP Tool Mapping

The MCP registry is scope-filtered. Local stdio discovery uses the local scope
set; remote HTTP discovery uses the validated bearer-token scopes after
intersection with `remote_mcp.allowed_scopes`. Preview, paper, and live tools
are visible only when their explicit scopes are granted, and the runtime gates
still run before any broker write boundary.

| Tool | Minimum scope |
|------|---------------|
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
| `ibkr_order_preview` | `ibkr:orders:preview` |
| `ibkr_paper_order_submit` | `ibkr:orders:paper:submit` |
| `ibkr_paper_order_cancel` | `ibkr:orders:paper:cancel` |
| `ibkr_live_order_submit` | `ibkr:orders:live:submit` |
| `ibkr_live_order_cancel` | `ibkr:orders:live:cancel` |

## Denials

Missing scope returns `AUTH_MISSING_SCOPE` and emits a denied-scope audit event.
Unknown local or remote scopes fail configuration validation.
