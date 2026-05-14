# Data Model: Local Sidecar Relay for Retail CP Gateway

## SidecarIdentity

**Fields**: `sidecar_id`, `public_key`, `created_at`, `display_name`, `capabilities`.

**Rules**: Private key stays local.

## PairingRecord

**Fields**: `pairing_id`, `remote_instance_id`, `sidecar_id`, `user_id`, `created_at`, `expires_at`, `status`.

**Rules**: Pairing is explicit and revocable.

## RelaySession

**Fields**: `relay_session_id`, `sidecar_id`, `remote_instance_id`, `heartbeat_at`, `expires_at`, `capabilities`.

**Rules**: Missing heartbeat fails closed.

## ForwardedBrokerRequest

**Fields**: `request_id`, `tool_name`, `scope`, `payload_hash`, `created_at`.

**Rules**: No broker cookies, raw headers, local paths, or credentials are forwarded to MCP clients.
