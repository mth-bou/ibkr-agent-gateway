# Contract: Local Scopes

The read-only MVP uses local configuration scopes. This is not OAuth/OIDC. Remote bearer token validation is reserved for the future remote MCP spec.

## Required Read Scopes

- `ibkr:health:read`
- `ibkr:accounts:read`
- `ibkr:portfolio:read`
- `ibkr:positions:read`
- `ibkr:marketdata:read`
- `ibkr:orders:read`
- `ibkr:audit:read`

## Rules

- Every MCP tool maps to exactly one minimum scope.
- Missing or insufficient scope denies before broker access.
- Scope denials emit `tool.denied_scope`.
- Write scopes are invalid in this MVP.
- OAuth/OIDC fields such as issuer, audience, expiry, token introspection, and JWKS validation are out of scope here.
