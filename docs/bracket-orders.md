# Bracket Orders

Bracket workflows are explicit grouped-order tools. They do not use the generic
`ibkr_order_submit` or `ibkr_order_modify` names.

| Tool | Scope |
|------|-------|
| `ibkr_bracket_order_preview` | `ibkr:orders:preview` |
| `ibkr_paper_bracket_order_submit` | `ibkr:orders:paper:submit` |
| `ibkr_live_bracket_order_submit` | `ibkr:orders:live:submit` |

The preview tool creates three non-executable legs: parent entry, take-profit
limit, and stop-loss stop. Submit tools reload server-persisted approvals and
previews before calling the configured group writer.

Live bracket submit uses the same live gates as single-order live writes:
enabled live config, allowlisted account, live scope, open kill switch, audit
availability, live limit checks for each leg, and paper-to-live
acknowledgement. The MCP handler also inserts durable pending idempotency state
before the writer boundary, persists the successful replay payload, and marks
all three approvals consumed after success.

The bundled MCP wiring delegates each live bracket leg to the configured
`LiveOrderWriter` through `SequentialLiveOrderGroupWriter`. That gives the same
writer boundary as live submit, but it is not broker-native OCA atomicity. A
deployment that requires native IBKR bracket/OCA behavior should add a
dedicated `LiveOrderGroupWriter` implementation and validate it in paper before
enabling live trading.
