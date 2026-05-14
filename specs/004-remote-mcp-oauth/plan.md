# Implementation Plan: Remote MCP OAuth/OIDC Front-Door

**Branch**: `004-remote-mcp-oauth` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

## Summary

Adds HTTP MCP transport and OAuth/OIDC-protected-resource behavior for remote clients. It authenticates MCP clients to the gateway; it does not replace IBKR backend authentication.

## Crates Touched

- `ibkr-domain`
- `ibkr-auth`
- `ibkr-audit`
- `ibkr-backend`
- `ibkr-mcp`
- `ibkr-cli`

Additional crates may be introduced only when listed in this plan's tasks.

## Constitution Check

- Provider-neutral broker core remains intact.
- Auth/scopes/audit are applied before broker calls.
- Unsupported or disabled behavior fails closed.
- All new schemas are typed and covered by contract tests.
