# Testing

The read-only MVP is validated offline through Cargo-discoverable tests under
`tests/` and fake Client Portal Gateway fixtures under `tests/fixtures/cpapi/`.

## Required Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI also runs:

```bash
cargo doc --workspace --no-deps
```

## Fixture Coverage

The fixture suite covers:

- session usable, missing, expired, keepalive success, and keepalive expiry
- accounts list
- portfolio snapshot and positions
- stock/ETF contract search and ambiguity
- live, delayed, and stale market snapshots
- historical bars
- read-only orders, order status, and executions

Fixtures must not contain tokens, cookies, credentials, sensitive headers, local
secret paths, bearer values, or raw broker session material.

## Replay and Performance

Replay tests check redaction and secret-scan behavior. Performance tests assert
local fake backend calls stay below the MVP gateway overhead budget and
in-memory audit appends stay below the audit write budget.

To measure the full offline suite duration locally:

```bash
time cargo test --workspace
```

The target from the spec is under 30 seconds for the complete offline fixture
suite.
