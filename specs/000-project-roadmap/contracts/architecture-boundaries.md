# Contract: Architecture Boundaries

## Core Rule

Broker logic, risk logic, order lifecycle logic, audit logic, and authorization logic must remain deterministic Rust code. LLM/provider output is data, never executable policy.

## Crate Boundaries

| Crate | Owns | Must Not Own |
|------|------|--------------|
| `ibkr-domain` | pure value types, identifiers, errors, money, account/contract/order models | HTTP, MCP, OAuth, storage, config loading, provider SDKs |
| `ibkr-cpapi` | Client Portal Gateway HTTP adapter models and calls | MCP handlers, risk policies, provider SDKs |
| `ibkr-backend` | backend trait, fake backend, backend factory | concrete MCP transport, OAuth JWT parsing |
| `ibkr-config` | runtime config loading and validation | broker secrets beyond secret wrappers, domain business logic |
| `ibkr-auth` | local scopes and auth context facade | IBKR broker session cookies, order risk decisions |
| `ibkr-audit` | event model, redaction, HMAC, persistence | broker calls, LLM prompts as policy |
| `ibkr-mcp` | MCP schemas, registry, scope guard, transports | broker protocol details, provider-specific business logic |
| `ibkr-cli` | operator commands | broker internals beyond service calls |
| `ibkr-risk` | deterministic risk checks | MCP transport, provider-specific prompting |
| `ibkr-orders` | preview/submit/cancel lifecycle | auth token parsing, MCP transport |
| `ibkr-oauth` | OAuth/OIDC validation | broker order decisions |
| `ibkr-sidecar` | relay protocol and sidecar session | remote OAuth policy beyond delegated checks |
| `ibkr-provider-compat` | compatibility tests/snapshots | production broker execution logic |
