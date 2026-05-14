# Contract: Order Preview Phase

## New Tools

- CLI: `ibkr-agent orders preview ...`
- MCP: `ibkr_order_preview`

## Required Scopes

- `ibkr:orders:preview`
- `ibkr:risk:read`

## Required Events

- `order.intent.received`
- `order.risk.checked`
- `order.preview.created`
- `order.preview.refused`

## Forbidden

- `ibkr_order_submit`
- `ibkr_order_cancel`
- `ibkr_order_approve`
- any broker-side order creation
