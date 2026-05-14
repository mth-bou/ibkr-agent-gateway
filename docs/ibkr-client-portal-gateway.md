# Interactive Brokers Client Portal Gateway

Retail Client Portal Gateway authentication is handled outside this project by
the local Interactive Brokers gateway process.

The local read-only MVP must report whether the broker session is usable,
requires manual action, expired, or unavailable. It must not return broker
cookies, credentials, raw headers, local secret paths, or raw session material to
CLI, MCP, logs, or audit records.
