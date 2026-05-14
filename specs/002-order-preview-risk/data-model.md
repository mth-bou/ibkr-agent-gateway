# Data Model: Order Preview and Deterministic Risk Engine

## OrderIntent

Non-executable proposal created by a user or agent.

**Fields**: `intent_id`, `account_id`, `contract_query_or_id`, `side`, `quantity`, `order_type`, `limit_price`, `time_in_force`, `rationale`, `created_by`, `created_at`.

**Rules**: Missing account, contract, side, quantity, order type, or time-in-force fails closed. Free text can explain rationale but cannot define executable fields.

## RiskPolicy

Deterministic constraints for preview.

**Fields**: `policy_id`, `enabled`, `allowed_account_modes`, `allowed_asset_classes`, `max_notional`, `max_quantity`, `max_order_count_per_window`, `allow_fractional`, `requires_market_data_freshness`, `requires_approval_for_preview`.

**Rules**: Preview is disabled by default until this feature is enabled. Policy changes are audited.

## RiskCheckResult

Output of deterministic validation.

**Fields**: `decision`, `warnings`, `refusals`, `estimated_notional`, `policy_id`, `checked_at`, `audit_event_id`.

**Rules**: Any refusal prevents preview creation.

## ValidatedOrder

A normalized broker-order candidate eligible only for preview.

**Fields**: `validated_order_id`, `intent_id`, `account_id`, `contract_id`, `side`, `quantity`, `order_type`, `limit_price`, `time_in_force`, `expires_at`, `warnings`.

**Rules**: Expiration prevents reuse. Validation does not imply approval or submission.

## OrderPreview

Non-executable preview result.

**Fields**: `preview_id`, `validated_order_id`, `estimated_cost`, `estimated_commission`, `margin_impact`, `warnings`, `expires_at`, `audit_event_id`.

**Rules**: Preview cannot call submit/cancel endpoints and cannot create broker-side orders.
