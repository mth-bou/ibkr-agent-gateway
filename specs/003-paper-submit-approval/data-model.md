# Data Model: Paper Submit and Approval Workflow

## ApprovalRecord

Explicit permission to submit/cancel a paper order.

**Fields**: `approval_id`, `preview_id`, `approved_by`, `method`, `decision`, `expires_at`, `created_at`, `audit_event_id`.

**Rules**: Single-use, expires, and binds to exactly one preview.

## IdempotencyKey

Prevents duplicate broker writes.

**Fields**: `key`, `operation`, `account_id_hash`, `request_hash`, `created_at`, `status`.

**Rules**: Same key and same request returns previous result; same key and different request refuses.

## SubmittedOrder

Paper broker order submission result.

**Fields**: `submitted_order_id`, `broker_order_id`, `account_id`, `preview_id`, `approval_id`, `idempotency_key`, `submitted_at`, `status`.

**Rules**: Account mode must be `paper`.

## OrderLifecycleEvent

Broker state transition for paper orders.

**Fields**: `event_id`, `broker_order_id`, `state`, `filled_quantity`, `remaining_quantity`, `timestamp`, `source`.

**Rules**: Unknown states are captured and surfaced as warnings, not dropped.
