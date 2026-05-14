# Implementation Plan: Local Sidecar Relay for Retail CP Gateway

**Branch**: `005-sidecar-relay` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

## Summary

Adds a local sidecar that bridges a retail user’s local CP Gateway to a remote MCP gateway without automating IBKR login or exposing session material to MCP clients.

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
