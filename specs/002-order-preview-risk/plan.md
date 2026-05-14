# Implementation Plan: Order Preview and Deterministic Risk Engine

**Branch**: `002-order-preview-risk` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

## Summary

Add the first write-adjacent capability: typed order intent, deterministic risk checks, validated order creation, and order preview. This phase still cannot submit, cancel, approve, or create broker-side orders.

## Technical Context

**New crates**: `crates/ibkr-risk/` and `crates/ibkr-orders/`.

**Updated crates**: `ibkr-domain`, `ibkr-config`, `ibkr-audit`, `ibkr-mcp`, `ibkr-cli`, `ibkr-backend`, `ibkr-cpapi`.

**Storage**: SQLite audit only; preview records may be stored locally only if needed for expiration/idempotency preparation.

**Testing**: Risk unit tests, preview fixture tests, MCP/CLI schema tests, forbidden submit/cancel tests, audit tests.

## Constraints

- No submit endpoint.
- No cancel endpoint.
- No approval workflow.
- No paper or live execution.
- Preview must be disabled by default in config.
- Risk policy must fail closed.
