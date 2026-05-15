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

Example configuration shape:

```yaml
gateway:
  mode: remote_mcp
  bind: 0.0.0.0:8080

remote_mcp:
  enabled: true
  resource: https://gateway.example.com/mcp
  issuer: https://auth.example.com/
  jwks_url: https://auth.example.com/.well-known/jwks.json
  metadata_url: https://auth.example.com/.well-known/openid-configuration
  audiences:
    - https://gateway.example.com/mcp
  allowed_scopes:
    - ibkr:health:read
    - ibkr:accounts:read
    - ibkr:portfolio:read
    - ibkr:positions:read
    - ibkr:marketdata:read
    - ibkr:orders:read
    - ibkr:audit:read
  clock_skew_seconds: 60

safety:
  remote_public_mcp_enabled: true
```

Request behavior:

- missing, malformed, expired, wrong issuer, wrong audience, or bad signature:
  `401` with `WWW-Authenticate` and protected-resource metadata
- valid token with missing tool scope: `403`
- valid token with the required scope: the request is authorized and the token
  is not included in downstream broker calls or audit payloads

This implementation validates HS256 JWTs against JWKS `oct` keys for
deterministic local and CI coverage. Adding RS256/ES256 provider keys should be
done inside `ibkr-oauth` without changing broker-core crates or tool schemas.
