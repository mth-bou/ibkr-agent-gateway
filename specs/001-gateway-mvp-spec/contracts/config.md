# Contract: Local Gateway Configuration

Configuration controls local read-only behavior. The MVP must reject settings that attempt to enable write tools, remote public mode, sidecar relay, direct broker OAuth2, or live trading writes.

Configuration is owned by the runtime configuration/application layer (`ibkr-config` in the implementation plan), not by `ibkr-domain`. Domain crates may expose pure value types consumed by config validation, but they must not own storage DSNs, file paths, or runtime loading behavior.

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

market_data:
  max_snapshot_age_seconds: 900
  allow_delayed: true
  stale_policy: warn
  missing_timestamp_policy: refuse

auth:
  mode: local_config
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
  account_id_mode: hmac
  hmac_key_source: local_generated

safety:
  write_tools_enabled: false
  remote_public_mcp_enabled: false
  sidecar_enabled: false
  direct_broker_oauth_enabled: false
  live_trading_enabled: false
```

## Validation Rules

- `server.mode` must be `local`.
- `broker.backend` must be `client_portal_gateway` or `fake`.
- `auth.mode` must be `local_config`.
- `safety.write_tools_enabled` must be `false`.
- `safety.remote_public_mcp_enabled` must be `false`.
- `safety.sidecar_enabled` must be `false`.
- `safety.direct_broker_oauth_enabled` must be `false`.
- `safety.live_trading_enabled` must be `false`.
- Only read scopes are valid in `auth.enabled_scopes`.
- Missing broker base URL returns `CONFIG_MISSING_BROKER_BASE_URL` before serving tools.
- `verify_tls=false` is valid only when `base_url` host is `localhost`, `127.0.0.1`, or `::1`.
- `market_data.max_snapshot_age_seconds` must be positive.
- `market_data.stale_policy` must be `warn` or `refuse`.
- `audit.account_id_mode` defaults to `hmac`; raw mode is invalid in this MVP.
