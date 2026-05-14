# Data Model: Live Trading Gated Enablement

## LiveTradingGate

**Fields**: `enabled`, `account_allowlist`, `required_scopes`, `approval_policy`, `risk_policy_id`, `audit_required`, `kill_switch_required`.

**Rules**: Disabled by default.

## KillSwitch

**Fields**: `state`, `changed_by`, `changed_at`, `reason`, `audit_event_id`.

**Rules**: Closed kill switch prevents live submit/cancel.

## LiveLimitPolicy

**Fields**: `max_notional`, `max_quantity`, `allowed_symbols`, `allowed_asset_classes`, `frequency_limits`, `session_limits`.

**Rules**: Any missing or failed limit refuses live writes.

## LiveSubmissionAttempt

**Fields**: `attempt_id`, `approved_order_id`, `idempotency_key`, `account_id`, `risk_result`, `kill_switch_state`, `submitted_at`.

**Rules**: Requires all gates and audit availability.
