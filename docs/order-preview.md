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

The MCP registry advertises `ibkr_order_preview` when
`ibkr:orders:preview` is present in the active local scope set or remote bearer
token grant. The tool uses the same validation and risk path as the CLI and
returns a persisted `preview_id` for later approval-bound paper or live flows.
The persisted validated order includes the resolved contract id, symbol, and
asset class so live allowlist checks evaluate the actual previewed instrument.

MCP preview accepts `limit`, `stop`, `stop_limit`, and `trailing_stop`
candidates. Limit candidates require `limit_price`; stop candidates require
`stop_price`; stop-limit candidates require both; trailing-stop candidates
require `trailing_amount` or `trailing_percent`. Market candidates remain
refused by the default deterministic policy.

Generic submit, cancel, approve, and modify tool names remain forbidden; use
the explicit preview, paper, and live-gated tools.

## Risk Policy

Risk checks are deterministic. The default policy is disabled and therefore
fails closed. Enabled policy checks currently cover account mode, asset class,
positive quantity, fractional quantity, quantity limit, order type, required
price fields for the selected order type, trailing offsets, and estimated
notional.

## Audit

Preview work emits preview-phase audit shapes:

- `order.intent.received`
- `order.risk.checked`
- `order.preview.created`
- `order.preview.refused`

Audit payloads must stay redacted and must not contain broker session material,
tokens, cookies, credentials, sensitive headers, or local secret paths.
