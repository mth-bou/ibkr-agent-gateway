# Remote MCP OAuth/OIDC

Remote MCP is disabled by default. Enabling it requires both the remote MCP
configuration block and the explicit safety flag, so an incomplete deployment
fails closed instead of exposing unauthenticated broker tools.

The CLI YAML config uses `safety.remote_mcp_enabled`. The public Rust
`GatewayConfiguration` struct exposes the equivalent SDK-facing field as
`safety.remote_public_mcp_enabled`.

Minimum configuration fields:

- `remote_mcp.enabled: true`
- `remote_mcp.resource`: public protected-resource identifier for the gateway
- `remote_mcp.issuer`: expected OIDC issuer
- `remote_mcp.jwks_url`: JWKS endpoint used for token signature checks
- `remote_mcp.audiences`: accepted token audiences/resources
- `remote_mcp.allowed_scopes`: gateway scopes that may be granted remotely
- `remote_mcp.token_id_hmac_secret_env`: environment variable containing the
  deployment secret used to hash token ids for audit correlation
- `safety.remote_mcp_enabled: true`

The CLI HTTP transport is functional when `--describe` is omitted:

```bash
ibkr-agent --config config/remote.example.yaml mcp serve --transport http --enable-remote-mcp --bind 0.0.0.0:8080
```

It binds the configured address, serves protected-resource metadata at
`/.well-known/oauth-protected-resource`, and accepts JSON-RPC MCP requests at
`POST /mcp`. The implementation intentionally uses a small bounded HTTP/1.1
server: it validates `Content-Length`, caps headers and bodies, returns one
response per connection, and routes authorized requests through the same
tool handlers as stdio.

Broker authentication remains separate from MCP client authorization; MCP
bearer tokens must never be forwarded to IBKR.

Example configuration shape:

```yaml
gateway:
  mode: remote_mcp
  bind: 0.0.0.0:8080

remote_mcp:
  enabled: true
  bind_address: 0.0.0.0:8080
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
  token_id_hmac_secret_env: IBKR_REMOTE_TOKEN_HMAC_SECRET

safety:
  remote_mcp_enabled: true
```

Request behavior:

- missing, malformed, expired, wrong issuer, wrong audience, or bad signature:
  `401` with `WWW-Authenticate` and protected-resource metadata
- valid token with missing tool scope: `403`
- repeated authorization attempts from the same forwarded IP or MCP session:
  `429`
- valid token with the required scope: the request is authorized and the token
  is not included in downstream broker calls or audit payloads
- `initialize` and `tools/list` validate the bearer token without a single
  required tool scope; visible tools are filtered to the token scopes allowed
  by `remote_mcp.allowed_scopes`
- `tools/call` requires the called tool scope before any backend, order writer,
  or audit workflow access

Production builds validate RS256 JWTs against RSA JWKS keys. HS256 and JWKS
`oct` key material are compiled only with the `unstable-internal-test-support`
feature for deterministic local and CI coverage. Adding ES256 provider keys
should stay inside the OAuth verifier without changing broker-core crates or
tool schemas.
