# Contract: Provider Compatibility

Provider compatibility is a test and packaging layer. It must not alter broker, risk, order, or audit semantics.

## Targets

- local MCP inspector
- Cursor/Continue MCP clients
- OpenAI remote MCP connector/tools
- Anthropic MCP connector/tools

## Required Evidence

- tool discovery snapshots
- schema snapshots
- auth denial snapshots
- successful read-only tool call snapshots
- order preview snapshots once `002` exists
- paper submit refusal/approval snapshots once `003` exists
- redaction/secret scanning snapshots

## Rules

- No OpenAI/Anthropic SDK dependency in `ibkr-domain`, `ibkr-backend`, `ibkr-risk`, or `ibkr-orders`.
- Provider adapters may live in `ibkr-provider-compat` or examples.
- Provider differences are handled by MCP-compatible schema/error design, not by broker logic forks.
