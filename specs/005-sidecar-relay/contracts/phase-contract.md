# Contract: Sidecar Relay Phase

## Required Gates

- remote MCP OAuth already active
- explicit sidecar pairing
- mutually authenticated relay session
- heartbeat valid
- local Client Portal Gateway session usable

## Forbidden

- automating retail IBKR browser login
- exposing local CP Gateway publicly
- sending broker cookies/session material to MCP clients
- continuing broker calls after heartbeat/session loss
