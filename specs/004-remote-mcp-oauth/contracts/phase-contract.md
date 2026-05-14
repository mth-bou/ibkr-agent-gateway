# Contract: Remote MCP OAuth/OIDC Phase

## Required Checks

- bearer token present
- signature valid
- issuer allowed
- audience/resource allowed
- expiry valid
- required scope present

## Denial Rules

- missing/invalid/expired token: 401
- valid token with insufficient scope: 403
- all denials audited with redacted token metadata

## Forbidden

- forwarding MCP bearer tokens to IBKR
- treating broker session as MCP authorization
- remote unauthenticated MCP
