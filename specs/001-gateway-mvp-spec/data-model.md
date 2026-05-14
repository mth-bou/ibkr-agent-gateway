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

## BrokerSessionStatus

Represents whether the broker backend can serve read-only data.

**Fields**

- `status`: `usable`, `manual_action_required`, `unavailable`
- `backend`: `client_portal_gateway`
- `checked_at`: timestamp
- `user_action`: optional safe instruction for manual recovery
- `error_code`: optional typed error code

**Validation Rules**

- Must not include cookies, tokens, sensitive headers, local credential paths, or
  raw broker session material.
- `manual_action_required` must include a safe user action.

## BrokerAccount

Represents an account visible to the current user/session.

**Fields**

- `account_id`: broker account identifier
- `account_label`: optional safe display label
- `account_mode`: `paper`, `live`, or `unknown`
- `base_currency`: currency code when known
- `metadata_redacted`: boolean

**Validation Rules**

- `account_id` is required for account-specific read requests.
- If multiple accounts are available and none is selected, account-specific
  requests fail closed.

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
- Position data without currency is flagged as incomplete or refused depending
  on the source response.

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
- A single resolved contract requires unambiguous symbol, asset class, currency,
  and exchange context.

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
- `staleness`

**Validation Rules**

- Snapshot must carry currency and timestamps.
- Stale or unavailable data returns a warning or structured refusal rather than
  pretending the value is current.

## ReadOnlyOrderRecord

Represents existing order, status, or execution data that can be viewed but not
changed.

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
- Submit, cancel, preview, and modification operations are invalid states.

## AuditEvent

Represents an append-only record of allowed or denied gateway behavior.

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
- `decision`
- `error_code`
- `input_hash`
- `output_hash`
- `redactions`
- `metadata`

**Validation Rules**

- Tokens, cookies, credentials, sensitive headers, and local secret paths are
  never stored raw.
- Significant operations produce an audit event even when denied.

## GatewayConfiguration

Represents local configuration for read-only operation.

**Fields**

- `server_mode`: `local`
- `bind_address`
- `broker_backend`: `client_portal_gateway`
- `client_portal_base_url`
- `audit_storage`
- `enabled_read_scopes`
- `redaction_policy`
- `write_tools_enabled`: always `false` for this feature

**Validation Rules**

- Non-local server modes are invalid for this feature.
- Write tools cannot be enabled by configuration in this feature.

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

Each transition emits `backend.session.changed` when the externally visible
state changes.

### Tool Call

```text
received -> scope_checked -> denied_scope
received -> scope_checked -> validated -> backend_called -> completed
received -> scope_checked -> validated -> refused
received -> scope_checked -> validated -> backend_called -> failed
```

Every terminal state emits an audit event.
