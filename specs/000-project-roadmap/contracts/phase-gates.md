# Contract: Phase Gates

## Global Gate

No phase may weaken the safety guarantees of a prior phase.

## Phase 001 Completion Gate

- zero write tools exposed
- 100% write-like refusals audited
- local scopes enforced
- audit HMAC redaction in place
- Client Portal Gateway keepalive handled
- fake backend fixture suite passes

## Phase 002 Completion Gate

- no broker submit/cancel endpoint used
- order preview deterministic
- unsupported/ambiguous/stale cases fail closed
- risk policies tested with fixtures

## Phase 003 Completion Gate

- paper-only submit/cancel
- explicit approval artifact required
- idempotency tested
- order lifecycle audit present

## Phase 004 Completion Gate

- remote auth denies invalid tokens before tool execution
- scopes and audience tested
- no IBKR secret exposed to client

## Phase 005 Completion Gate

- sidecar relay mutually authenticated
- local Client Portal Gateway not publicly exposed
- heartbeat loss fails closed

## Phase 006 Completion Gate

- provider compatibility snapshots pass
- no provider-specific broker logic introduced

## Phase 007 Completion Gate

- live disabled by default
- live allowlist, approval, risk, idempotency, audit, and kill switch all required
- paper/live separation tests pass

