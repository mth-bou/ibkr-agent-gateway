# Contract: MCP Tool Maturity Expansion

This contract records the current MCP surface and the target additions for the
maturity expansion. All tools are scope-filtered. Missing scope must hide the
tool from discovery and return `AUTH_MISSING_SCOPE` if invoked directly through
an internal path.

## Current Tools

| Tool | Scope | Input summary | Notes |
|------|-------|---------------|-------|
| `ibkr_health` | `ibkr:health:read` | `{}` | Gateway liveness |
| `ibkr_backend_status` | `ibkr:health:read` | `{}` | Broker session status |
| `ibkr_session_requirements` | `ibkr:health:read` | `{}` | Manual login/session requirements |
| `ibkr_accounts_list` | `ibkr:accounts:read` | `{}` | Safe visible accounts |
| `ibkr_account_summary` | `ibkr:portfolio:read` | `account_id` | Broker summary payload |
| `ibkr_positions_list` | `ibkr:positions:read` | `account_id` | Open positions |
| `ibkr_portfolio_snapshot` | `ibkr:portfolio:read` | `account_id` | Aggregated portfolio |
| `ibkr_contracts_search` | `ibkr:marketdata:read` | `query` | Contract candidates |
| `ibkr_contract_resolve` | `ibkr:marketdata:read` | `symbol` | One resolved contract or refusal |
| `ibkr_market_snapshot` | `ibkr:marketdata:read` | `contract_id` | Quote snapshot |
| `ibkr_historical_bars` | `ibkr:marketdata:read` | `contract_id`, `duration`, `bar_size` | OHLCV bars |
| `ibkr_orders_list` | `ibkr:orders:read` | `account_id` | Current/open read-only orders |
| `ibkr_order_status` | `ibkr:orders:read` | `account_id`, `broker_order_id` | One order status |
| `ibkr_executions_list` | `ibkr:orders:read` | `account_id` | Executions |
| `ibkr_audit_tail` | `ibkr:audit:read` | `limit` | Redacted verified audit tail |
| `ibkr_order_preview` | `ibkr:orders:preview` | account, symbol, side, quantity, limit order fields | Non-executable preview |
| `ibkr_paper_order_submit` | `ibkr:orders:paper:submit` | `account_id`, `approval_id`, `idempotency_key` | Server loads preview and approval |
| `ibkr_paper_order_cancel` | `ibkr:orders:paper:cancel` | `account_id`, `broker_order_id`, `idempotency_key` | Paper cancel |
| `ibkr_live_order_submit` | `ibkr:orders:live:submit` | `account_id`, `approval_id`, `preview_id`, `idempotency_key` | Server loads all live gates |
| `ibkr_live_order_cancel` | `ibkr:orders:live:cancel` | `account_id`, `broker_order_id`, `idempotency_key` | Live-gated cancel |

## Forbidden Generic Names

These names must remain forbidden unless a future accepted spec explicitly
changes the naming policy:

- `ibkr_order_intent_validate`
- `ibkr_order_preview_explain`
- `ibkr_order_submit`
- `ibkr_order_cancel`
- `ibkr_order_modify`
- `ibkr_order_approve`

## Phase 1 Target Tools

| Tool | Scope | Required input | Output contract |
|------|-------|----------------|-----------------|
| `ibkr_pnl_daily` | `ibkr:portfolio:read` | `account_id` | `{ account_id_hash?, realized_pnl, unrealized_pnl, total_pnl, currency, period_start, period_end, timestamp, data_status }` |
| `ibkr_pnl_realtime` | `ibkr:portfolio:read` | `account_id` | `{ account_id_hash?, rows[], total_pnl, currency, timestamp, data_status }` |
| `ibkr_orders_history` | `ibkr:orders:read` | `account_id`, optional `from`, optional `to`, optional `status`, optional `limit` | bounded historical order records |
| `ibkr_account_metadata` | `ibkr:accounts:read` | `account_id` | safe account mode, base currency, margin/cash status, product permissions, restrictions |
| `ibkr_kill_switch_status` | `ibkr:health:read` | `{}` | current live kill switch state and safe reason |
| `ibkr_limits_status` | `ibkr:risk:read` | `account_id`, optional `policy_id` | live limit counters and remaining budget |
| `ibkr_audit_export` | `ibkr:audit:export` | optional `limit`, optional `format=jsonl` | redacted export metadata and JSONL payload |
| `ibkr_session_renew` | `ibkr:health:read` | `{}` | keepalive result session status |

## Phase 2 Target Tools

| Tool | Scope | Required input | Output contract |
|------|-------|----------------|-----------------|
| `ibkr_paper_order_modify` | `ibkr:orders:paper:modify` | `account_id`, `broker_order_id`, `idempotency_key`, bounded changes | modified paper lifecycle |
| `ibkr_live_order_modify` | `ibkr:orders:live:modify` | `account_id`, `broker_order_id`, `idempotency_key`, bounded changes, optional `approval_id` depending policy | modified live lifecycle |

Bounded changes may include:

- `limit_price`
- `stop_price`
- `quantity`
- `time_in_force`
- `trailing_amount`
- `trailing_percent`

The handler must reject any attempt to replace account, contract, side, or
broker order identity from MCP payload.

## Phase 3 Target Tools

| Tool | Scope | Required input | Output contract |
|------|-------|----------------|-----------------|
| `ibkr_options_chain` | `ibkr:options:read` | `underlying_contract_id` or explicit underlying query, optional expiry/strike/right filters | bounded option contracts and entitlement status |
| `ibkr_option_greeks` | `ibkr:options:read` | `option_contract_id` | delta/gamma/theta/vega/IV and timestamp |
| `ibkr_market_depth` | `ibkr:marketdata:depth:read` | `contract_id`, optional `levels` | bounded bid/ask book |
| `ibkr_scanner_run` | `ibkr:scanner:read` | allowlisted `scanner_code`, optional filters, optional `limit` | bounded scanner rows |

## Phase 4 Target Tools

| Tool | Scope | Required input | Output contract |
|------|-------|----------------|-----------------|
| `ibkr_bracket_order_preview` | `ibkr:orders:preview` | parent entry plus take-profit and stop-loss definitions | non-executable group preview |
| `ibkr_paper_bracket_order_submit` | `ibkr:orders:paper:submit` | `account_id`, `approval_id`, `group_preview_id`, `idempotency_key` | paper group lifecycle |
| `ibkr_live_bracket_order_submit` | `ibkr:orders:live:submit` | `account_id`, `approval_id`, `group_preview_id`, `idempotency_key` | live-gated group lifecycle |
| `ibkr_oca_group_preview` | `ibkr:orders:preview` | OCA legs | non-executable OCA preview |
| `ibkr_paper_oca_group_submit` | `ibkr:orders:paper:submit` | `account_id`, `approval_id`, `group_preview_id`, `idempotency_key` | paper OCA lifecycle |
| `ibkr_live_oca_group_submit` | `ibkr:orders:live:submit` | `account_id`, `approval_id`, `group_preview_id`, `idempotency_key` | live-gated OCA lifecycle |

## Phase 5 Target Tools

| Tool | Scope | Required input | Output contract |
|------|-------|----------------|-----------------|
| `ibkr_news_list` | `ibkr:news:read` | `contract_id` or symbol query, optional `limit` | bounded article metadata |
| `ibkr_news_article` | `ibkr:news:read` | `article_id` | bounded article text treated as untrusted content |
| `ibkr_fundamentals_get` | `ibkr:fundamentals:read` | `contract_id`, optional report type | bounded fundamentals payload |
| `ibkr_market_holidays` | `ibkr:calendar:read` | exchange or calendar code, optional date range | holiday/session closures |
| `ibkr_market_session` | `ibkr:calendar:read` | exchange or contract, timestamp | open/closed/tradable status |
| `ibkr_currency_rate` | `ibkr:currency:read` | `base_currency`, `quote_currency` | FX rate, timestamp, source |
| `ibkr_transfer_history` | `ibkr:transfers:read` | `account_id`, optional date range, optional limit | deposits/withdrawals safely redacted |
| `ibkr_approvals_create` | `ibkr:approvals:create` | `account_id`, `preview_id`, `ttl_seconds` | persisted approval record |

## Safety Requirements

- Every response must be serializable through the safe output schema.
- Every audit event must include tool name, scope, result status, error code
  when applicable, correlation id, and redaction metadata.
- Account ids in audit must be HMAC-hashed, not stored raw.
- News, scanner labels, issuer names, and article text must be treated as
  untrusted external text.
- Write-capable tools must use idempotency records and pending recovery context
  before calling a broker writer.
