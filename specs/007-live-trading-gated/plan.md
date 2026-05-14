# Implementation Plan: Live Trading Gated Enablement

**Branch**: `007-live-trading-gated` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

## Summary

Adds live trading only after read-only, preview/risk, paper submit, remote auth, sidecar, and provider compatibility are stable. Live is disabled by default and gated by multiple independent controls.

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
