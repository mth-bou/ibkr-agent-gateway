# Order Preview

Order preview is write-adjacent but still non-executable: the gateway can
validate an intent and produce a preview, but it cannot submit, cancel, approve,
modify, or create a broker-side order.

## CLI

Preview is disabled by default. A local run must opt in explicitly:

```bash
ibkr-agent orders preview \
  --account DU1234567 \
  --symbol AAPL \
  --side buy \
  --quantity 1 \
  --limit-price 100 \
  --currency USD \
  --enable-preview \
  --json
```

Without `--enable-preview`, the command returns `ORDER_PREVIEW_DISABLED`.

## MCP

The production stdio MCP registry is read-only and does not advertise
`ibkr_order_preview` by default. Preview remains available through the explicit
CLI and SDK workflow. Submit, cancel, approve, and modify tools remain absent or
refused.

## Risk Policy

Risk checks are deterministic. The default policy is disabled and therefore
fails closed. Enabled policy checks currently cover account mode, asset class,
positive quantity, fractional quantity, quantity limit, order type, limit price,
and estimated notional.

## Audit

Preview work emits preview-phase audit shapes:

- `order.intent.received`
- `order.risk.checked`
- `order.preview.created`
- `order.preview.refused`

Audit payloads must stay redacted and must not contain broker session material,
tokens, cookies, credentials, sensitive headers, or local secret paths.
