# Quickstart: Order Preview and Deterministic Risk Engine

1. Complete `001-gateway-mvp-spec`.
2. Enable preview in config while leaving submit/cancel disabled.
3. Run risk unit tests and preview fixtures.
4. Call CLI `ibkr-agent orders preview ...` and MCP `ibkr_order_preview`.
5. Verify submit/cancel tool names remain absent or refused.
6. Review audit events for intent, risk decision, preview, and refusals.
