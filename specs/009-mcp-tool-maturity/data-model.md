# Data Model: MCP Tool Maturity Expansion

## PnlSnapshot

**Fields**: `account_id`, `period`, `realized_pnl`, `unrealized_pnl`,
`total_pnl`, `currency`, `timestamp`, `data_status`.

**Validation**:

- `account_id` must be a typed `AccountId`.
- Money values must use `Money`.
- `data_status` must distinguish live, delayed, stale, and unavailable.

## HistoricalOrderRecord

**Fields**: `account_id`, `broker_order_id`, `contract_id`, `symbol`,
`asset_class`, `side`, `quantity`, `order_type`, `limit_price`, `stop_price`,
`trailing_amount`, `trailing_percent`, `time_in_force`, `status`,
`submitted_at`, `updated_at`, `filled_quantity`, `average_fill_price`,
`cancel_reason`.

**Validation**:

- Date ranges must be bounded by config.
- Result size must be bounded.
- Account ids in audit must be HMAC-hashed.

## AccountCapabilityProfile

**Fields**: `account_id`, `account_mode`, `base_currency`, `cash_or_margin`,
`product_permissions`, `option_level`, `shorting_allowed`, `pdt_status`,
`gfv_status`, `restrictions`, `metadata_redacted`.

**Validation**:

- Only safe account capability metadata may be exposed.
- Raw broker session, cookies, tokens, and credential-like fields are forbidden.

## KillSwitchStatus

**Fields**: `state`, `changed_by`, `changed_at`, `reason`, `audit_event_id`.

**State transitions**:

- Status tool is read-only.
- Existing CLI/operator flows may open or close the switch.

## LimitsStatus

**Fields**: `account_id`, `policy_id`, `submitted_in_session`,
`submitted_in_window`, `session_notional`, `remaining_session_orders`,
`remaining_window_orders`, `remaining_session_notional`, `currency`,
`window_seconds`, `timestamp`.

**Validation**:

- Values must be derived from trusted local audit/idempotency state.
- Missing policy must return a structured refusal or empty policy status, not a
  fabricated budget.

## OrderModifyIntent

**Fields**: `account_id`, `broker_order_id`, `idempotency_key`, `approval_id`,
`limit_price`, `stop_price`, `quantity`, `time_in_force`, `trailing_amount`,
`trailing_percent`, `created_by`, `created_at`.

**Validation**:

- Account, broker order id, and idempotency key are required.
- Account, contract, side, and broker order identity cannot be replaced.
- At least one bounded change must be present.
- Live modify must pass live gates and kill switch checks.

## AdvancedOrderType

**Variants**:

- `limit`: requires `limit_price`.
- `market`: no price, refused by default for live.
- `stop`: requires `stop_price`.
- `stop_limit`: requires `stop_price` and `limit_price`.
- `trailing_stop`: requires exactly one of `trailing_amount` or
  `trailing_percent`.

## OptionChain

**Fields**: `underlying_contract_id`, `expiration`, `strike`, `right`,
`option_contract_id`, `symbol`, `exchange`, `currency`, `data_status`.

**Validation**:

- Result rows must be bounded.
- Unsupported option entitlements must produce structured refusal.

## OptionGreeks

**Fields**: `option_contract_id`, `delta`, `gamma`, `theta`, `vega`,
`implied_volatility`, `model_source`, `timestamp`, `data_status`.

## MarketDepthBook

**Fields**: `contract_id`, `bids`, `asks`, `levels`, `timestamp`,
`data_status`.

**Depth row fields**: `level`, `venue`, `price`, `size`.

**Validation**:

- Requested levels must be bounded by config.

## ScannerRunResult

**Fields**: `scanner_code`, `filters`, `rows`, `timestamp`,
`snapshot_timestamp`.

**Validation**:

- Scanner code must come from an allowlist.
- Result limit must be bounded.

## BracketOrderGroup

**Fields**: `group_id`, `account_id`, `parent_order`, `take_profit`,
`stop_loss`, `expires_at`, `warnings`.

**State transitions**:

1. Three leg previews created under one `group_id`.
2. Approvals created for each leg preview.
3. Pending submit record inserted.
4. Broker group writer called.
5. Group lifecycle completed or failed-after-writer.

OCA group ids and broker-native OCA atomics are future scope, not part of the
implemented Spec 009 bracket model.

## NewsArticle

**Fields**: `article_id`, `provider`, `headline`, `published_at`,
`related_contracts`, `body`, `truncated`.

**Validation**:

- Body must be bounded.
- Body is untrusted external text and must not be rendered into audit metadata
  as instructions.

## FundamentalsReport

**Fields**: `contract_id`, `report_type`, `as_of`, `payload`, `truncated`.

## MarketSessionStatus

**Fields**: `exchange`, `contract_id`, `timestamp`, `is_open`,
`is_tradable`, `next_open`, `next_close`, `reason`.

## CurrencyRate

**Fields**: `base_currency`, `quote_currency`, `rate`, `timestamp`, `source`.

## TransferRecord

**Fields**: `account_id`, `transfer_id`, `type`, `amount`, `currency`,
`status`, `created_at`, `settled_at`.

**Validation**:

- Counterparty/bank details must be redacted or omitted.

## ApprovalCreateRequest

**Fields**: `account_id`, `preview_id`, `ttl_seconds`, `approved_by`.

**Validation**:

- Preview must exist and match account.
- TTL must be within configured bounds.
- Approval creation is a gateway workflow write, not a broker write.
