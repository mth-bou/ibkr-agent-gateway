# Sidecar Relay

The sidecar relay lets a remote MCP gateway route authorized broker requests to
a local Client Portal Gateway without exposing IBKR session material remotely.

Required gates:

- remote MCP OAuth is already enabled and validates the MCP client token
- the local sidecar has an explicit pairing record for the remote instance
- the relay session is bound to the paired sidecar id
- heartbeat is valid and the relay session has not expired
- the local Client Portal Gateway session is usable

The sidecar does not automate retail IBKR browser login. If the local Client
Portal Gateway requires authentication, the remote gateway must return a manual
local action requirement instead of attempting login automation.

Sensitive material is not forwarded to MCP clients. Forwarded broker requests
carry the tool name, required scope, request id, and a payload hash. Cookies,
authorization headers, tokens, credentials, secrets, and local filesystem paths
are refused before forwarding.

Typical local flow:

```bash
ibkr-agent sidecar identity create --display-name laptop --json
ibkr-agent sidecar pairing create \
  --remote-instance-id remote-1 \
  --sidecar-id sidecar-example \
  --user-id local-user \
  --ttl-seconds 300 \
  --json
```

Configuration remains disabled by default and needs both `sidecar.enabled` and
`safety.sidecar_enabled`. Sidecar relay also requires remote MCP to be enabled;
otherwise validation fails closed.
