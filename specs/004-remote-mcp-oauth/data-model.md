# Data Model: Remote MCP OAuth/OIDC Front-Door

## OAuthIssuerConfig

**Fields**: `issuer`, `jwks_url`, `audiences`, `allowed_scopes`, `clock_skew_seconds`, `metadata_url`.

**Rules**: Issuer and audience must match before scope checks.

## OAuthTokenClaims

**Fields**: `sub`, `iss`, `aud`, `exp`, `nbf`, `iat`, `scope`, `jti`, `tenant_id`.

**Rules**: Raw token is never stored; `jti` is HMAC-hashed in audit if stored.

## RemoteAuthContext

**Fields**: `user_id`, `subject`, `issuer`, `audience`, `scopes`, `expires_at`, `token_id_hash`.

**Rules**: Missing/invalid/expired tokens deny before tool execution.

## HttpMcpSession

**Fields**: `session_id`, `auth_context`, `transport`, `created_at`, `last_seen_at`.

**Rules**: Does not hold broker credentials.
