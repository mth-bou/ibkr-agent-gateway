# Contract: Operations Hardening

## Required Capabilities

- audit export
- replay fixture generation and replay
- safe metrics
- structured logs
- retention and backup rules
- schema drift detection
- incident review workflow

## Forbidden Behavior

- raw broker sessions in exports
- account IDs in metric labels
- tokens/cookies/credentials in logs
- live broker dependency for replay

