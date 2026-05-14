# Contract: Live Trading Gated Phase

## Required Gates

- live feature explicitly enabled
- live account allowlist
- live write scope
- unexpired preview
- approval record
- idempotency key
- live risk policy pass
- kill switch open
- audit storage available
- paper-to-live checklist acknowledged

## Forbidden

- live enabled by default
- live submit while any gate is missing
- live submit with closed kill switch
- live submit without audit availability
