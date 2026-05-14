# Research: Operations Hardening

## Decision: Use redacted replay, not raw broker sessions

**Rationale**: Replay must be CI-safe and must not require live broker auth or raw secrets.

## Decision: Metrics labels must be non-sensitive

**Rationale**: Observability systems often have broad access. Account IDs, tokens, local paths, and broker session material cannot appear in labels or logs.

