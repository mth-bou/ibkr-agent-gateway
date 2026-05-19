# Bracket Orders

Bracket workflows are explicit grouped-order tools. They do not use the generic
`ibkr_order_submit` or `ibkr_order_modify` names.

| Tool | Scope |
|------|-------|
| `ibkr_bracket_order_preview` | `ibkr:orders:preview` |
| `ibkr_paper_bracket_order_submit` | `ibkr:orders:paper:submit` |
| `ibkr_live_bracket_order_submit` | `ibkr:orders:live:submit` |

The preview tool creates three non-executable legs: parent entry, take-profit
limit, and stop-loss stop. The gateway persists the bracket preview as one
server-owned group record. Submit tools reload the three approvals, reload the
matching persisted group by its leg preview ids, and reject approvals mixed from
different bracket previews before any writer call.

Paper bracket submit uses the same durable idempotency boundary as other MCP
order writes: it checks for a persisted replay payload first, writes pending
idempotency state before calling the writer, persists the successful replay
payload, and marks all three approvals consumed. Repeating the same request
with the same idempotency key replays the original payload even after those
approvals are consumed.

Live bracket submit uses the same live gates as single-order live writes:
enabled live config, allowlisted account, live scope, open kill switch, audit
availability, live limit checks for each leg, and paper-to-live
acknowledgement. The MCP handler also inserts durable pending idempotency state
before the writer boundary, persists the successful replay payload, and marks
all three approvals consumed after success.

The bundled MCP wiring delegates each live bracket leg to the configured
`LiveOrderWriter` through `SequentialLiveOrderGroupWriter`. That gives the same
writer boundary as live submit, but it is not broker-native OCA atomicity. If a
later leg fails after earlier legs have already been submitted, the writer
returns an error that lists every already-submitted broker order id in the
message, `user_action`, and `tracing` log entry so operators can clean up the
orphaned legs before retrying. A deployment that requires native IBKR
bracket/OCA behavior should add a dedicated `LiveOrderGroupWriter`
implementation and validate it in paper before enabling live trading.
