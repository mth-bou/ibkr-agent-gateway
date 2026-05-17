# Documentation

This directory contains developer-facing usage, production, and operations
documentation.

## Start Here

- [Developer Guide](developer-guide.md): install, CLI, SDK, MCP, order workflows,
  audit, and validation commands.
- [Production Readiness](production-readiness.md): deployment checklist,
  secrets, remote MCP, audit, sidecar, paper, and live gates.
- [Public API](public-api.md): supported Rust facade modules and examples.
- [Testing](testing.md): local gates, fixture coverage, replay, and performance
  budgets.

## Feature Guides

- [Client Portal Gateway](ibkr-client-portal-gateway.md): local broker session
  expectations and troubleshooting.
- [Read and Tool Commands](tools.md): read-only data, preview, paper, live-gated,
  and sidecar CLI commands.
- [Local and Remote MCP](mcp-local.md): MCP transports, tool registry, forbidden
  generic write tools, and provider usage.
- [Remote MCP OAuth/OIDC](remote-mcp-oauth.md): protected resource metadata,
  RS256/JWKS validation, token-id hashing, and rate limiting.
- [Order Preview](order-preview.md): non-executable previews and deterministic
  risk checks.
- [Paper Orders](paper-orders.md): approval and idempotent paper lifecycle.
- [Paper to Live](paper-to-live.md) and [Live Runbook](live-runbook.md): live
  trading gates and operational checklist.
- [Sidecar Relay](sidecar-relay.md): pairing, heartbeat, and local CP Gateway
  forwarding boundary.
- [Provider Compatibility](provider-compatibility.md): MCP client targets and
  provider-neutral compatibility checks.
- [Audit Log](audit-log.md), [Audit Retention](audit-retention.md), and
  [Incident Review](incident-review.md): evidence handling and redacted review
  workflows.
