# Interactive Brokers Client Portal Gateway

Retail Client Portal Gateway authentication is handled outside this project by
the local Interactive Brokers gateway process.

The local gateway reports whether the broker session is usable, requires manual
action, expired, or unavailable. It must not return broker cookies,
credentials, raw headers, local secret paths, or raw session material to CLI,
MCP, logs, or audit records.

## Session States

Session and keepalive outcomes map into safe statuses:

- `usable`: read-only account discovery may proceed.
- `manual_action_required`: the user must complete or refresh broker login
  outside this gateway.
- `unavailable`: the local Client Portal Gateway cannot be reached.

The fake backend covers connected, missing-session, expired-session,
keepalive-success, and keepalive-expired fixtures so session behavior can be
validated offline before any live broker session is used.

## Contextual Read Endpoints

The contextual read adapter includes fixture-backed and wiremock-covered paths
for options, greeks, market depth, scanners, news, fundamentals,
calendar/session, FX, and transfer history. These tests lock the gateway client
contract only. Interactive Brokers can vary Client Portal Gateway endpoint
availability and naming by Gateway build and entitlement, so validate every
contextual read against the exact deployed IBKR Gateway before relying on it in
production.

## Troubleshooting

- If `backend status` reports manual action required, complete or refresh the
  broker login in the Interactive Brokers Client Portal Gateway UI.
- If keepalive fails, broker-backed read tools must be treated as unavailable
  until manual login is restored.
- If the local gateway is unreachable, check that the Client Portal Gateway
  process is running and that the configured URL is local.
- `verify_tls=false` is only valid for the exact hosts `localhost`,
  `127.0.0.1`, or `::1`; wider loopback ranges and IPv4-mapped IPv6 addresses
  are intentionally not treated as local by the config validator.
- Broker cookies, raw headers, tokens, credential file paths, and raw session
  material must never appear in CLI, MCP, logs, fixtures, or audit output.

For offline checks, run the fake fixture test suite with:

```bash
cargo test --workspace --features unstable-internal-test-support
```
