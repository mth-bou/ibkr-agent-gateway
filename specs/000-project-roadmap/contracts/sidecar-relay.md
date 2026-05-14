# Contract: Sidecar Relay

## Purpose

The sidecar relay allows a remote OAuth-protected MCP server to reach a local Client Portal Gateway session without exposing local broker secrets to remote MCP clients.

## Required Components

- local sidecar process;
- remote relay endpoint;
- mutually authenticated sidecar session;
- heartbeat;
- reconnect behavior;
- per-request correlation;
- audit events on both remote and local sides.

## Secret Rules

The sidecar must never send to MCP clients:

- IBKR session cookies;
- local broker headers;
- credentials;
- local filesystem paths;
- raw Client Portal Gateway session material.

## Failure Behavior

- Missing sidecar session: deny broker-backed tools with manual action.
- Expired broker session: deny broker-backed tools with manual action.
- Relay disconnect: fail closed and audit.
- Duplicate request id: return prior idempotent result when safe or refuse.
