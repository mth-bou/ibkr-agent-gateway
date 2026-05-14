# Implementation Plan: OpenAI, Anthropic, and MCP Client Compatibility

**Branch**: `006-provider-compatibility` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

## Summary

Adds compatibility harnesses for OpenAI Responses MCP, Anthropic MCP connector, Cursor, Continue, and local MCP clients. It does not add provider dependencies to broker core.

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
