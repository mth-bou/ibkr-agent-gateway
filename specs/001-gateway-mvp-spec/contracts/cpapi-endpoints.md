# Contract: Client Portal Gateway Endpoint Mapping

This contract maps internal gateway methods to Client Portal Gateway endpoint families. Exact endpoint paths and query parameters must be verified against the IBKR Client Portal API documentation during implementation, but MCP/CLI code must depend only on the internal methods below.

## Rules

- MCP and CLI handlers call `IbkrBackend`, not raw HTTP endpoints.
- `ibkr-cpapi` owns HTTP paths, response models, and broker-specific error mapping.
- `ibkr-backend` maps CPAPI models into domain models.
- All calls must redact cookies, headers, tokens, and raw session material from logs/errors/audit.
- Session/keepalive calls must update `BrokerSessionStatus`.

## Endpoint Families

| Internal Method | CPAPI Endpoint Family | Domain Output | Notes |
|----------------|-----------------------|---------------|-------|
| `health()` | gateway reachability / local health | gateway status | May be local adapter check, not broker data |
| `session_status()` | auth/session status | `BrokerSessionStatus` | Used for manual action decisions |
| `tickle()` | keepalive/tickle | `BrokerSessionStatus` | Used by long-running MCP server |
| `list_accounts()` | portfolio/accounts | `Vec<BrokerAccount>` | Safe metadata only |
| `account_summary(account_id)` | portfolio summary | `PortfolioSnapshot` summary fields | Account must be explicit |
| `positions(account_id)` | portfolio positions pages | `Vec<Position>` | Page through all available data |
| `portfolio_snapshot(account_id)` | portfolio/account allocation data | `PortfolioSnapshot` | May combine summary + positions |
| `contract_search(query, filters)` | security definition search | `Vec<ContractCandidate>` | MVP supports stock/ETF only |
| `contract_resolve(filters)` | security definition search/details | resolved contract or refusal | Must not silently pick ambiguous candidates |
| `market_snapshot(contract_id)` | market data snapshot | `MarketSnapshot` | Must include status/staleness/warnings |
| `historical_bars(contract_id, duration, bar_size)` | market data history | `Vec<HistoricalBar>` | Refuse unsupported cases |
| `orders_list(account_id, status)` | account orders | `Vec<ReadOnlyOrderRecord>` | Read-only only |
| `order_status(account_id, broker_order_id)` | order status | `ReadOnlyOrderRecord` | Read-only only |
| `executions_list(account_id, range)` | executions/trades | execution records | Read-only only |

## Required CPAPI Error Mapping

- unauthenticated/expired session -> `BROKER_SESSION_REQUIRED`
- gateway unavailable -> `BROKER_BACKEND_UNAVAILABLE`
- broker rate limit -> `BROKER_RATE_LIMITED`
- unsupported endpoint/data -> `BROKER_CAPABILITY_UNAVAILABLE`
- malformed broker response -> `BROKER_RESPONSE_INVALID`
- missing currency/timestamp -> `MARKET_DATA_INCOMPLETE` or structured refusal
- ambiguous contract -> `INPUT_AMBIGUOUS_CONTRACT`
- unsupported asset class -> `INPUT_UNSUPPORTED_ASSET_CLASS`
