# Provider Compatibility

Provider compatibility is a test and configuration layer around MCP. It does
not add provider SDKs to broker, risk, auth, audit, backend, or MCP core crates,
and it does not change broker execution semantics.

Supported targets:

- generic MCP inspector over stdio
- Cursor MCP client over stdio
- Continue MCP client over stdio
- OpenAI remote MCP over HTTP with OAuth bearer auth
- Anthropic MCP connector over HTTP with OAuth bearer auth

Local clients use the same stdio command shape:

```bash
ibkr-agent mcp serve --transport stdio
```

Example client configuration files are available under `examples/mcp-clients/`:

- `generic-inspector.json`
- `cursor.json`
- `continue.json`

These examples reference `IBKR_CONFIG` only. They must not embed IBKR
credentials, Client Portal Gateway cookies, OAuth tokens, refresh tokens,
broker session ids, or local absolute paths.

Remote provider connectors should use the remote MCP HTTP endpoint documented
in `docs/remote-mcp-oauth.md`. The provider receives the gateway protected
resource metadata and sends an OAuth bearer token for the MCP request. The
gateway validates issuer, audience, expiry, signature, and tool scope before
any broker access. MCP bearer tokens are never forwarded to IBKR and are never
written to audit payloads.

Provider-specific behavior belongs in `crates/ibkr-provider-compat/` or
examples. Core crates stay provider-neutral; compatibility is proven through:

- schema snapshots generated from the broker MCP tool registry
- auth denial snapshots for missing token and missing scope
- redaction snapshots for provider-visible outputs and example configs
- dependency checks that forbid provider SDK dependencies in production crates

The provider compatibility harness currently exercises representative read-only
flows and denial behavior. Order preview, paper order, and live trading provider
flows remain governed by their own roadmap specs and must not bypass gateway
policy, approval, scope, or audit gates.
