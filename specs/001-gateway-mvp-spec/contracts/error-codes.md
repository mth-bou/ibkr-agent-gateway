# Contract: Error Codes

Errors returned by CLI, MCP, and backend services must use stable codes. Messages must be user-actionable and must not include secrets.

## Configuration

| Code | Retryable | User Action |
|------|-----------|-------------|
| `CONFIG_INVALID` | false | Fix the config file |
| `CONFIG_MISSING_BROKER_BASE_URL` | false | Configure broker base URL |
| `CONFIG_TLS_BYPASS_NON_LOCALHOST` | false | Enable TLS verification or use localhost |
| `CONFIG_WRITE_TOOLS_FORBIDDEN` | false | Remove write-tool config |
| `CONFIG_REMOTE_MCP_FORBIDDEN` | false | Disable remote MCP for MVP |
| `CONFIG_SIDECAR_FORBIDDEN` | false | Disable sidecar for MVP |
| `CONFIG_LIVE_TRADING_FORBIDDEN` | false | Disable live trading for MVP |

## Authorization

| Code | Retryable | User Action |
|------|-----------|-------------|
| `AUTH_MISSING_SCOPE` | false | Enable required local read scope |
| `AUTH_SCOPE_NOT_ALLOWED_IN_MVP` | false | Remove write/remote/live scope |
| `AUTH_LOCAL_ONLY_MVP` | false | Use local config auth in MVP |

## Input and Ambiguity

| Code | Retryable | User Action |
|------|-----------|-------------|
| `INPUT_MISSING_ACCOUNT` | false | Select an account explicitly |
| `INPUT_UNAUTHORIZED_ACCOUNT` | false | Select an accessible account |
| `INPUT_AMBIGUOUS_ACCOUNT` | false | Select one account explicitly |
| `INPUT_AMBIGUOUS_CONTRACT` | false | Provide symbol, asset class, currency, and exchange |
| `INPUT_UNSUPPORTED_ASSET_CLASS` | false | Use stock/ETF in MVP |
| `INPUT_INVALID_CONTRACT` | false | Use a resolved contract id |
| `INPUT_INVALID_TIME_RANGE` | false | Provide a valid time range |

## Broker Backend

| Code | Retryable | User Action |
|------|-----------|-------------|
| `BROKER_SESSION_REQUIRED` | true | Complete broker login manually |
| `BROKER_SESSION_EXPIRED` | true | Reauthenticate broker session |
| `BROKER_BACKEND_UNAVAILABLE` | true | Start or check Client Portal Gateway |
| `BROKER_RATE_LIMITED` | true | Retry later |
| `BROKER_CAPABILITY_UNAVAILABLE` | false | Use a supported read-only capability |
| `BROKER_RESPONSE_INVALID` | true | Retry or inspect broker response safely |

## Market Data

| Code | Retryable | User Action |
|------|-----------|-------------|
| `MARKET_DATA_UNAVAILABLE` | true | Check market data permissions or retry |
| `MARKET_DATA_DELAYED` | false | Treat value as delayed |
| `MARKET_DATA_STALE` | false | Refresh or adjust stale policy |
| `MARKET_DATA_INCOMPLETE` | true | Retry or inspect source availability |
| `HISTORICAL_BARS_UNAVAILABLE` | true | Adjust duration/bar size or retry |

## Read-Only Boundary

| Code | Retryable | User Action |
|------|-----------|-------------|
| `READONLY_WRITE_FORBIDDEN` | false | Use a later feature spec for preview/trading |
| `READONLY_ORDER_PREVIEW_FORBIDDEN` | false | Use spec 002 after implementation |
| `READONLY_ORDER_SUBMIT_FORBIDDEN` | false | Use spec 003/007 after implementation |
| `READONLY_ORDER_CANCEL_FORBIDDEN` | false | Use spec 003/007 after implementation |

## Output and Audit

| Code | Retryable | User Action |
|------|-----------|-------------|
| `OUTPUT_UNSAFE` | false | Inspect redacted output or adjust source |
| `AUDIT_WRITE_FAILED` | true | Fix audit storage before relying on the gateway |
| `AUDIT_READ_FORBIDDEN` | false | Enable `ibkr:audit:read` |
