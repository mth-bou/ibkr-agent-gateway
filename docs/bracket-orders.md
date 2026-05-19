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
previews before calling the configured group writer. Live bracket submit uses
the same live gates as single-order live writes: enabled live config,
allowlisted account, live scope, open kill switch, audit availability, and
paper-to-live acknowledgement.
