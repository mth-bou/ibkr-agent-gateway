# Contract: Project Scopes

Scopes are the minimum authorization unit for MCP tools and CLI/API surfaces. A tool maps to exactly one minimum scope unless a later spec explicitly documents an additional policy gate. Missing the minimum scope always denies before broker access.

## Phase 001 Read Scopes

- `ibkr:health:read`
- `ibkr:accounts:read`
- `ibkr:portfolio:read`
- `ibkr:positions:read`
- `ibkr:marketdata:read`
- `ibkr:orders:read`
- `ibkr:audit:read`

## Phase 002 Preview/Risk Scopes

- `ibkr:orders:preview`
- `ibkr:risk:read`

## Phase 003 Paper Write Scopes

- `ibkr:orders:paper:submit`
- `ibkr:orders:paper:cancel`
- `ibkr:approvals:create`
- `ibkr:approvals:read`

## Phase 004 Remote Auth Requirements

Remote tokens map to the same scopes. Remote MCP additionally checks:

- issuer
- audience/resource
- expiry
- signature
- subject/user binding
- tenant binding when configured
- token revocation when configured

## Phase 005 Sidecar Scopes

- `ibkr:sidecar:pair`
- `ibkr:sidecar:read`
- `ibkr:sidecar:relay`

## Phase 007 Live Write Scopes

- `ibkr:orders:live:submit`
- `ibkr:orders:live:cancel`
- `ibkr:live:gates:read`
- `ibkr:live:killswitch:write`

Live scopes are invalid unless live trading is explicitly implemented, configured, account-allowlisted, risk-gated, approval-gated, idempotency-gated, audit-gated, and kill-switch-open.
