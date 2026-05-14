# Implementation Plan: Operations, Export, Replay, and Hardening

**Branch**: `008-operations-hardening` | **Date**: 2026-05-14 | **Spec**: [spec.md](./spec.md)

## Summary

Add operational tooling after all safety-critical broker and MCP features exist. Focus on audit export, replay, safe metrics, structured logs, incident review, retention, backups, dependency scanning, and contract drift detection.

## Dependencies

- Phase 001 read-only MVP complete.
- Phase 002 preview/risk complete.
- Phase 003 paper submit/approval complete.
- Phase 004 remote MCP OAuth complete.
- Phase 005 sidecar complete.
- Phase 006 provider compatibility complete.
- Phase 007 live trading gated complete, if live trading has been enabled.

## Deliverables

- `audit export` CLI command.
- replay harness.
- metrics module.
- structured logs.
- schema drift tests.
- retention/backup docs.
- incident review docs.
- dependency/security CI gates.

