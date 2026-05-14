# Contract: Local Gateway Configuration

Configuration controls local read-only behavior. The MVP must reject settings
that attempt to enable write tools, remote public mode, sidecar relay, or live
trading writes.

## Example

```yaml
server:
  mode: local
  bind: "127.0.0.1:8080"

broker:
  backend: client_portal_gateway
  client_portal_gateway:
    base_url: "https://localhost:5000/v1/api"
    verify_tls: false
    keepalive_interval_seconds: 60

auth:
  local_user_id: "local-user"
  enabled_scopes:
    - ibkr:health:read
    - ibkr:accounts:read
    - ibkr:portfolio:read
    - ibkr:positions:read
    - ibkr:marketdata:read
    - ibkr:orders:read
    - ibkr:audit:read

audit:
  enabled: true
  storage: "sqlite://ibkr-agent.db"
  account_id_mode: hash

safety:
  write_tools_enabled: false
  remote_public_mcp_enabled: false
  sidecar_enabled: false
```

## Validation Rules

- `server.mode` must be `local`.
- `safety.write_tools_enabled` must be `false`.
- `safety.remote_public_mcp_enabled` must be `false`.
- `safety.sidecar_enabled` must be `false`.
- Only read scopes are valid in `auth.enabled_scopes`.
- Missing broker base URL returns a configuration error before serving tools.
