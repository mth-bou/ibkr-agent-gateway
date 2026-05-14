# Contract: Provider Compatibility Phase

## Rule

Provider compatibility is evidence, examples, and snapshots. It must not change broker semantics.

## Required Targets

- generic MCP inspector
- Cursor/Continue or equivalent local MCP client
- OpenAI remote MCP
- Anthropic remote MCP

## Forbidden

- provider SDK dependency in `ibkr-domain`, `ibkr-backend`, `ibkr-risk`, `ibkr-orders`, or `ibkr-audit`
- provider-specific broker behavior
