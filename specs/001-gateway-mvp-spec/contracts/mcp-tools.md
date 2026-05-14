# Contract: Local Read-Only MCP Tools

All tools are read-only in this feature. No tool may preview, submit, cancel, or
modify an order. Tool results must not include tokens, cookies, credentials,
sensitive headers, local secret paths, or raw broker session material.

## Common Error Shape

```json
{
  "code": "AUTH_MISSING_SCOPE",
  "message": "Missing required scope: ibkr:accounts:read",
  "retryable": false,
  "user_action": null,
  "audit_event_id": "018f-example"
}
```

## Tools

| Tool | Scope | Input | Output |
|------|-------|-------|--------|
| `ibkr_health` | `ibkr:health:read` | `{}` | broker and gateway status summary |
| `ibkr_backend_status` | `ibkr:health:read` | `{}` | Client Portal Gateway status without secrets |
| `ibkr_session_requirements` | `ibkr:health:read` | `{}` | safe manual actions required to restore a session |
| `ibkr_accounts_list` | `ibkr:accounts:read` | `{}` | accessible accounts with safe metadata |
| `ibkr_account_summary` | `ibkr:portfolio:read` | `{ "account_id": "U1234567" }` | account cash, equity, margin, and base currency |
| `ibkr_positions_list` | `ibkr:positions:read` | `{ "account_id": "U1234567" }` | positions for one selected account |
| `ibkr_portfolio_snapshot` | `ibkr:portfolio:read` | `{ "account_id": "U1234567" }` | portfolio summary and allocations |
| `ibkr_contracts_search` | `ibkr:marketdata:read` | `{ "query": "AAPL", "asset_class": "stock", "currency": "USD", "exchange": "SMART" }` | candidate contracts |
| `ibkr_contract_resolve` | `ibkr:marketdata:read` | `{ "symbol": "AAPL", "asset_class": "stock", "currency": "USD", "exchange": "SMART" }` | one resolved contract or ambiguity refusal |
| `ibkr_market_snapshot` | `ibkr:marketdata:read` | `{ "contract_id": "265598" }` | bid, ask, last, currency, and timestamp |
| `ibkr_historical_bars` | `ibkr:marketdata:read` | `{ "contract_id": "265598", "duration": "1 D", "bar_size": "5 mins" }` | historical read-only bars when available |
| `ibkr_orders_list` | `ibkr:orders:read` | `{ "account_id": "U1234567", "status": "open" }` | open or recent orders |
| `ibkr_order_status` | `ibkr:orders:read` | `{ "account_id": "U1234567", "broker_order_id": "123" }` | read-only order status |
| `ibkr_executions_list` | `ibkr:orders:read` | `{ "account_id": "U1234567", "from": "2026-05-14T00:00:00Z", "to": "2026-05-14T23:59:59Z" }` | read-only execution records |
| `ibkr_audit_tail` | `ibkr:audit:read` | `{ "limit": 100 }` | recent redacted audit events |

## Explicitly Forbidden Tool Names

The local read-only MVP must not expose these tools:

- `ibkr_order_intent_validate`
- `ibkr_order_preview`
- `ibkr_order_preview_explain`
- `ibkr_order_submit`
- `ibkr_order_cancel`

Calls to forbidden tools must return a typed refusal and emit an audit event.
