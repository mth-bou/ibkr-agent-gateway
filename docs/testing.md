# Testing

The project is validated through Cargo-discoverable tests, fake Client Portal
Gateway fixtures, replay checks, provider snapshots, and local performance
budgets.

## Required Local Gates

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
```

CI also runs documentation and security workflows.

## Fixture Coverage

Fake CPAPI fixtures under `tests/fixtures/cpapi/` cover:

- session usable, missing, expired, keepalive success, and keepalive expiry;
- accounts list;
- portfolio snapshot and positions;
- stock/ETF contract search and ambiguity;
- live, delayed, and stale market snapshots;
- historical bars;
- read-only orders, order status, and executions.

Fixtures must not contain tokens, cookies, credentials, sensitive headers, local
secret paths, bearer values, or raw broker session material.

## Feature Coverage

The test suite covers:

- CLI contracts for read commands, audit, preview, paper, and live-gated
  refusals;
- MCP tool discovery, schemas, redaction, keepalive, and scope denials;
- remote OAuth RS256 validation, token redaction, generic auth denials,
  configurable rate limiting, and connection-cap handling;
- order preview, risk checks, paper approval/idempotency, live limits, kill
  switch, and paper-to-live gates;
- sidecar identity, pairing, heartbeat, forwarding safety, and secret scans;
- provider compatibility snapshots and provider SDK dependency boundaries.

## Replay and Performance

Replay tests check audit redaction and secret-scan behavior. Performance tests
assert budgets for fake backend reads, audit append/tail, cached remote OAuth
validation, prepared remote MCP authorization, live gate/risk/idempotency, and
sidecar request safety.

To measure the full offline suite duration locally:

```bash
time cargo test --workspace --features unstable-internal-test-support
```
