# Interactive Brokers Client Portal Gateway

Retail Client Portal Gateway authentication is handled outside this project by
the local Interactive Brokers gateway process.

The local read-only MVP must report whether the broker session is usable,
requires manual action, expired, or unavailable. It must not return broker
cookies, credentials, raw headers, local secret paths, or raw session material to
CLI, MCP, logs, or audit records.

## US1 Session States

The read-only MVP maps session and keepalive outcomes into safe statuses:

- `usable`: read-only account discovery may proceed.
- `manual_action_required`: the user must complete or refresh broker login
  outside this gateway.
- `unavailable`: the local Client Portal Gateway cannot be reached.

The fake backend covers connected, missing-session, expired-session,
keepalive-success, and keepalive-expired fixtures so US1 can be validated
offline before any live broker session is used.
