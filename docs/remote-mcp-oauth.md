# Remote MCP OAuth/OIDC

Remote MCP is disabled by default. Enabling it requires both the remote MCP
configuration block and the explicit safety flag, so an incomplete deployment
fails closed instead of exposing unauthenticated broker tools.

Minimum configuration fields:

- `remote_mcp.enabled: true`
- `remote_mcp.resource`: public protected-resource identifier for the gateway
- `remote_mcp.issuer`: expected OIDC issuer
- `remote_mcp.jwks_url`: JWKS endpoint used for token signature checks
- `remote_mcp.audiences`: accepted token audiences/resources
- `remote_mcp.allowed_scopes`: gateway scopes that may be granted remotely
- `safety.remote_public_mcp_enabled: true`

The HTTP transport exposes `/.well-known/oauth-protected-resource` metadata and
the `/mcp` endpoint. Broker authentication remains separate from MCP client
authorization; MCP bearer tokens must never be forwarded to IBKR.
