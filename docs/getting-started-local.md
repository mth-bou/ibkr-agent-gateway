# Getting Started: Local Read-Only Gateway

This guide tracks `specs/001-gateway-mvp-spec`.

The first implementation phase is local, single-user, and read-only. It targets
a manually authenticated Interactive Brokers Client Portal Gateway session and a
fake backend for offline validation.

## Initial Checks

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
```

Order preview, order submit, order cancel, remote MCP, sidecar relay, and live
trading remain out of scope for this phase.
