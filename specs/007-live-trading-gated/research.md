# Research: Live Trading Gated Enablement

## Decision Rules

- Do not introduce provider coupling into broker core.
- Do not introduce write behavior unless this spec explicitly allows it.
- Keep audit and scope behavior compatible with the local read-only MVP.
- Keep IBKR broker authentication separate from MCP client authorization.
