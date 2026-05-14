# Implementation Plan: Paper Submit and Approval Workflow

**Branch**: `003-paper-submit-approval` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

## Summary

Adds approval records, idempotency, paper-only submit/cancel, and order lifecycle tracking. Live trading remains unavailable.

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
