# Data Model: IBKR Agent Gateway Read-Only MVP

## LocalUser

Represents the person running the local gateway.

**Fields**

- `user_id`: stable local identifier used for audit correlation
- `display_name`: optional local label
- `allowed_scopes`: set of read scopes granted for local tool calls

**Validation Rules**

- `user_id` is required for any audited operation.
- Write scopes are not valid for this feature.

## AuthContext

Represents local caller authorization in the MVP.

**Fields**

- `source`: `local_config`
- `user_id`
- `scopes`
- `request_id`
- `session_id`

**Validation Rules**

- `source` must be `local_config` in this feature.
- Bearer tokens, OAuth issuer, audience, and expiry are not part of this MVP.
- Missing scope fails closed and emits `tool.denied_scope`.

## BrokerSessionStatus

Represents whether the broker backend can serve read-only data.

**Fields**

- `status`: `usable`, `manual_action_required`, `unavailable`
- `backend`: `client_portal_gateway` or `fake`
- `checked_at`: timestamp
- `last_keepalive_at`: optional timestamp
- `user_action`: optional safe instruction for manual recovery
- `error_code`: optional typed error code

**Validation Rules**

- Must not include cookies, tokens, sensitive headers, local credential paths, or raw broker session material.
- `manual_action_required` must include a safe user action.
- Status checks emit `backend.session.checked`.
- Status changes emit `backend.session.changed` when the previous externally visible status is known.

## BrokerAccount

Represents an account visible to the current user/session.

**Fields**

- `account_id`: broker account identifier
- `account_id_hash`: HMAC-SHA256 account correlation value for audit
- `account_label`: optional safe display label
- `account_mode`: `paper`, `live`, or `unknown`
- `base_currency`: currency code when known
- `metadata_redacted`: boolean

**Validation Rules**

- `account_id` is required for account-specific read requests.
- If multiple accounts are available and none is selected, account-specific requests fail closed.
- Audit uses `account_id_hash` by default, not raw account id.

## PortfolioSnapshot

Represents a read-only account summary and allocation view.

**Fields**

- `account_id`
- `base_currency`
- `net_liquidation`
- `cash_balances`
- `margin_summary`
- `allocations`
- `as_of`
- `data_status`

**Validation Rules**

- Monetary values must carry currency.
- Snapshot must include an `as_of` timestamp or return a structured refusal.

## Position

Represents a current holding.

**Fields**

- `account_id`
- `contract_id`
- `symbol`
- `asset_class`
- `quantity`
- `market_price`
- `market_value`
- `currency`
- `unrealized_pnl`
- `as_of`

**Validation Rules**

- Quantity and money values use decimal-safe representation.
- Position data without currency is flagged as incomplete or refused depending on the source response.

## ContractCandidate

Represents a possible instrument returned by search.

**Fields**

- `contract_id`
- `symbol`
- `description`
- `asset_class`
- `exchange`
- `currency`
- `is_unique_match`

**Validation Rules**

- A tool may return multiple candidates.
- A single resolved contract requires unambiguous symbol, asset class, currency, and exchange context.
- Supported MVP asset classes are `stock` and `etf` only.
- Unsupported asset classes return `INPUT_UNSUPPORTED_ASSET_CLASS`.

## MarketDataPolicy

Represents local rules for market-data freshness and delayed data.

**Fields**

- `max_snapshot_age_seconds`
- `allow_delayed`
- `stale_policy`: `warn` or `refuse`
- `missing_timestamp_policy`: `refuse`

**Validation Rules**

- `max_snapshot_age_seconds` must be positive.
- Delayed data must be labeled when allowed.
- Missing timestamp defaults to refusal.

## MarketSnapshot

Represents current read-only market data for a resolved contract.

**Fields**

- `contract_id`
- `bid`
- `ask`
- `last`
- `currency`
- `source_timestamp`
- `received_at`
- `data_status`: `live`, `delayed`, `stale`, `unavailable`
- `staleness_seconds`
- `warnings`

**Validation Rules**

- Snapshot must carry currency and timestamps.
- Stale or unavailable data returns a warning or structured refusal according to `MarketDataPolicy`.

## HistoricalBarsRequest

Represents a read-only request for historical bars.

**Fields**

- `contract_id`
- `duration`
- `bar_size`
- `outside_regular_trading_hours`

**Validation Rules**

- Unsupported durations, bar sizes, contracts, or asset classes return structured refusals.
- Historical bars output includes data status and warnings.

## HistoricalBar

Represents one read-only historical bar.

**Fields**

- `timestamp`
- `open`
- `high`
- `low`
- `close`
- `volume`
- `currency`

**Validation Rules**

- OHLC values use decimal-safe representation.
- Timestamp and currency must be present or the output is refused.

## ReadOnlyOrderRecord

Represents existing order, status, or execution data that can be viewed but not changed.

**Fields**

- `account_id`
- `broker_order_id`
- `status`
- `side`
- `quantity`
- `filled_quantity`
- `contract_id`
- `limit_price`
- `currency`
- `created_at`
- `updated_at`

**Validation Rules**

- Records are read-only in this feature.
- Submit, cancel, preview, approve, and modification operations are invalid states.

## ErrorCode

Represents stable user-actionable error categories.

**Fields**

- `code`
- `message`
- `retryable`
- `user_action`
- `audit_event_id`

**Validation Rules**

- Codes must come from `contracts/error-codes.md`.
- Messages must not include secrets.

## AuditEvent

Represents an append-only record of allowed, denied, refused, failed, or completed gateway behavior.

**Fields**

- `event_id`
- `event_type`
- `timestamp`
- `user_id`
- `session_id`
- `request_id`
- `account_id_hash`
- `tool_name`
- `scopes`
- `decision`: `allow`, `deny`, `refuse`
- `result_status`: `called`, `completed`, `failed`, `refused`, `denied_scope`
- `error_code`
- `input_hash`
- `output_hash`
- `redactions`
- `metadata`

**Validation Rules**

- Tokens, cookies, credentials, sensitive headers, local paths, and backend session material are never stored raw.
- Account identifiers default to HMAC-SHA256.
- Significant operations produce an audit event even when denied or refused.

## AuditRecorder

Represents the shared service for writing audit events.

**Fields**

- `storage_backend`: `sqlite`
- `hmac_key_source`: `local_generated`, `env`, or `file`
- `redaction_policy`

**Validation Rules**

- US1, US2, US3, and US4 all use the same recorder.
- Audit tail/query is US4; record-only writing exists before US1.

## GatewayConfiguration

Represents local configuration for read-only operation. This entity is owned by the runtime configuration/application layer, not by `ibkr-domain`; the domain crate may expose pure value types used by configuration, but it must not own storage DSNs, file paths, or runtime loading behavior.

**Fields**

- `server_mode`: `local`
- `bind_address`
- `broker_backend`: `client_portal_gateway` or `fake`
- `client_portal_base_url`
- `verify_tls`
- `keepalive_interval_seconds`
- `audit_storage`
- `audit_hmac_key_source`
- `enabled_read_scopes`
- `redaction_policy`
- `market_data_policy`
- `write_tools_enabled`: always `false` for this feature
- `remote_public_mcp_enabled`: always `false` for this feature
- `sidecar_enabled`: always `false` for this feature

**Validation Rules**

- Non-local server modes are invalid for this feature.
- Write tools cannot be enabled by configuration in this feature.
- `verify_tls=false` is valid only for `localhost`, `127.0.0.1`, or `::1` URLs.
- Only read scopes are valid.

## State Transitions

### Broker Session

```text
unknown -> usable
unknown -> manual_action_required
unknown -> unavailable
usable -> manual_action_required
usable -> unavailable
manual_action_required -> usable
unavailable -> usable
```

Every check emits `backend.session.checked`. Each transition emits `backend.session.changed` when the externally visible state changes and the previous state is known.

### Tool Call

```text
received -> scope_checked -> denied_scope
received -> scope_checked -> validated -> refused
received -> scope_checked -> validated -> backend_called -> completed
received -> scope_checked -> validated -> backend_called -> failed
```

Every terminal state emits an audit event.
