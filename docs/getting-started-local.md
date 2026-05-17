# Getting Started Locally

This page is kept for the original MVP path. For the complete current package
guide, start with [developer-guide.md](developer-guide.md).

## Local Fake Backend

The fastest development path uses fake Client Portal Gateway fixtures:

```bash
cargo run --bin ibkr-agent -- health --json
cargo run --bin ibkr-agent -- backend status --json
cargo run --bin ibkr-agent -- session requirements --json
cargo run --bin ibkr-agent -- accounts list --json
```

When no `--config` path is provided, the CLI uses fixtures under
`tests/fixtures/cpapi/` and stores local audit/workflow state in a SQLite file
under the system temp directory.

## Read Commands

```bash
ibkr-agent account summary --account DU1234567 --json
ibkr-agent portfolio snapshot --account DU1234567 --json
ibkr-agent positions list --account DU1234567 --json
ibkr-agent contracts search AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent contracts resolve AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent market snapshot --contract-id 265598 --json
ibkr-agent market bars --contract-id 265598 --duration "1 D" --bar-size "5 mins" --json
ibkr-agent orders list --account DU1234567 --json
ibkr-agent orders status --account DU1234567 --broker-order-id 123 --json
ibkr-agent executions list --account DU1234567 --json
```

## Feature-Gated Workflows

Order preview, paper order lifecycle, remote MCP, sidecar relay, provider
compatibility, and live safety gates now exist in the package. They remain
fail-closed and require explicit flags or configuration.

Use:

- [order-preview.md](order-preview.md)
- [paper-orders.md](paper-orders.md)
- [remote-mcp-oauth.md](remote-mcp-oauth.md)
- [sidecar-relay.md](sidecar-relay.md)
- [live-runbook.md](live-runbook.md)

## Validation

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
```
