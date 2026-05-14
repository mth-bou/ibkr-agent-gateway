# Contract: Paper Submit and Approval Phase

## Required Scopes

- `ibkr:orders:paper:submit`
- `ibkr:orders:paper:cancel`
- `ibkr:approvals:create`
- `ibkr:approvals:read`

## Required Gates

- paper account mode
- prior unexpired preview
- approval record
- idempotency key
- audit availability
- broker session usable

## Forbidden

- live account submit/cancel
- submit without approval
- duplicate idempotency writes
- expired preview submit
