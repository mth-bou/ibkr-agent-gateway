# Local Scopes

The read-only MVP uses local configuration scopes. These scopes are not OAuth
claims and do not imply future remote MCP permissions.

## Scope Rules

- Every MCP broker tool maps to one minimum read scope.
- Missing scope denies before broker access.
- Write, remote, sidecar, paper-trading, and live-trading scopes are invalid in
  `specs/001-gateway-mvp-spec`.
- OAuth/OIDC issuer, audience, expiry, JWKS, introspection, and bearer-token
  validation are reserved for `specs/004-remote-mcp-oauth`.

## Tool Mapping

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

Audit review uses `ibkr:audit:read`.
