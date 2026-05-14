# Contract: Project Boundaries

## Broker Core

Allowed:
- Typed broker requests and responses
- Error mapping
- Fake backend
- CPAPI backend
- Future IBKR OAuth/TWS adapters

Forbidden:
- OpenAI SDK imports
- Anthropic SDK imports
- Prompt execution
- MCP transport details inside domain types

## MCP Layer

Allowed:
- Tool registry
- Schema conversion
- Scope enforcement
- Transport handling
- Tool-call audit

Forbidden:
- Broker session secrets in responses
- Direct prompt-to-order execution
- Provider-specific business rules

## Provider Compatibility

Allowed:
- Test harnesses
- Example configs
- Adapter examples

Forbidden:
- Provider logic required for broker correctness
- Provider SDK dependency in broker crates
