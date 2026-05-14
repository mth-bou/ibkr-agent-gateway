# Contract: Order Preview Models and Boundary

This contract covers the preview-only phase. It does not permit broker-side
order creation, submit, cancel, modify, approve, paper execution, or live
execution.

## OrderIntent

`OrderIntent` is a typed, non-executable proposal.

Required fields:

- `intent_id`
- `account_id`
- `account_mode`
- `contract`
- `side`
- `quantity`
- `order_type`
- `time_in_force`
- `created_by`
- `created_at`

Rules:

- Free text rationale is explanatory only and cannot define executable fields.
- Limit order preview requires `limit_price`.
- Missing account, contract, side, quantity, order type, or time-in-force fails
  closed.

## RiskPolicy

`RiskPolicy` is deterministic and disabled by default.

Required policy controls:

- `enabled`
- `allowed_account_modes`
- `allowed_asset_classes`
- `max_notional`
- `max_quantity`
- `allow_fractional`
- `requires_market_data_freshness`
- `requires_approval_for_preview`

Rules:

- Disabled policy refuses preview.
- Unsupported account mode, asset class, quantity, order type, missing price, or
  notional limit failure refuses preview.

## RiskCheckResult

Implemented as `RiskDecision`.

Allowed result:

- decision: `allow`
- warnings: zero or more `RiskWarning`

Refused result:

- decision: `refuse`
- refusals: one or more `RiskRefusal`

## ValidatedOrder

`ValidatedOrder` is a normalized broker-order candidate eligible only for
preview.

Rules:

- It must contain a resolved `contract_id`.
- It must preserve account, side, quantity, order type, price, and time in
  force.
- It expires and cannot be reused as an approval or submit token in this phase.

## OrderPreview

`OrderPreview` is a non-executable output.

Required fields:

- `preview_id`
- `validated_order_id`
- `estimated_cost`
- `estimated_commission`
- `margin_impact`
- `warnings`
- `expires_at`
- `audit_event_id`

Rules:

- Creating a preview must not call submit, cancel, modify, approve, or live
  trading endpoints.
- `ibkr_order_submit`, `ibkr_order_cancel`, and `ibkr_order_approve` remain
  forbidden.
