# Research: Order Preview and Deterministic Risk Engine

## Decision: Preview is separate from submit

**Rationale**: This prevents a convenient preview flow from quietly becoming an execution flow.

## Decision: LLM creates `OrderIntent`, Rust validates

**Rationale**: The model can propose structured data, but deterministic Rust code owns validation and refusal.

## Decision: Risk policy is config-driven and audited

**Rationale**: Users need transparent and replayable reasons for warnings/refusals.
