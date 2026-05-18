# Sidecar Relay

The sidecar relay lets a remote MCP gateway route authorized broker requests to
a local Client Portal Gateway without exposing IBKR session material remotely.

Required gates for a remote deployment:

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
ibkr-agent sidecar identity create --public-key <public-key> --display-name laptop --json
ibkr-agent sidecar pairing create \
  --remote-instance-id remote-1 \
  --sidecar-id sidecar-example \
  --user-id local-user \
  --ttl-seconds 300 \
  --json
ibkr-agent sidecar session create \
  --remote-instance-id remote-1 \
  --sidecar-id sidecar-example \
  --ttl-seconds 300 \
  --json
ibkr-agent sidecar relay accept \
  --remote-instance-id remote-1 \
  --sidecar-id sidecar-example \
  --tool-name ibkr_accounts_list \
  --scope ibkr:accounts:read \
  --payload-json '{}' \
  --json
```

The CLI currently exposes relay primitives for operator and integration
workflows: identity creation, pairing records, relay sessions, and request
sanitization. It is not a long-running sidecar daemon and it does not persist or
heartbeat a local relay process by itself.

The SDK/config layer still models sidecar enablement as fail-closed. A remote
deployment should require both sidecar configuration and the independent safety
flag before accepting relay traffic, and it should also require remote MCP OAuth
to be enabled.
