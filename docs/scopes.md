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
| `ibkr:orders:live:submit` | live-gated submit candidate |
| `ibkr:orders:live:cancel` | live-gated cancel candidate |

Preview, paper, and live scopes do not bypass feature flags, approvals,
idempotency, risk limits, kill switch, audit availability, or migration
checklists.

## MCP Tool Mapping

The production stdio registry exposes read-only tools plus audit tail. Preview,
paper, and live workflows keep their scopes for CLI/SDK gates, but they are not
advertised by default local MCP discovery.

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

## Denials

Missing scope returns `AUTH_MISSING_SCOPE` and emits a denied-scope audit event.
Unknown local or remote scopes fail configuration validation.
