# Packaging Refactor Plan

Date: 2026-05-15

## Goal

Turn `ibkr-agent-gateway` into the single user-facing package for both:

- `cargo install ibkr-agent-gateway`, installing the `ibkr-agent` binary;
- `cargo add ibkr-agent-gateway`, exposing a small SDK-style API for embedding the gateway in another Rust project.

The repository may stay modular internally, but crates.io should present one product package, not a constellation of internal `ibkr-*` crates.

## Original State Before Refactor

- The root package `ibkr-agent-gateway` is a test harness and has `publish = false`.
- The actual binary is in `crates/ibkr-cli` as `ibkr-agent`.
- Production code is split across internal workspace crates such as `ibkr-domain`, `ibkr-auth`, `ibkr-mcp`, `ibkr-orders`, and `ibkr-sidecar`.
- Those internal crates currently depend on each other by `path`.
- Publishing only `ibkr-agent-gateway` is not viable while the root package depends on unpublished path crates. Cargo can use `path` locally, but published packages must resolve dependencies from the registry unless the code is part of the same package.

## Final State

- The root package `ibkr-agent-gateway` is the only Cargo workspace member.
- The installable binary is `src/bin/ibkr-agent.rs`.
- The SDK entrypoint is `src/lib.rs`, with stable public facades under `src/public/*`.
- Internal implementation is preserved as root package modules under `src/internal/*`.
- No `crates/ibkr-*` package or path dependency remains in the build graph.
- `publish = false` has been removed after the release candidate gates passed locally.
- The package has not been uploaded to crates.io yet; only `cargo publish --dry-run --locked` has been run.

## Architecture Decision

Move from "workspace crates as package boundaries" to "workspace modules as internal architecture" for the published product.

Target shape:

```text
ibkr-agent-gateway/
├── Cargo.toml                 # publishable package
├── src/
│   ├── lib.rs                 # public SDK facade
│   ├── bin/ibkr-agent.rs      # installable CLI binary
│   ├── cli/                   # former ibkr-cli internals
│   ├── internal/
│   │   ├── approval/
│   │   ├── audit/
│   │   ├── auth/
│   │   ├── backend/
│   │   ├── config/
│   │   ├── cpapi/
│   │   ├── domain/
│   │   ├── mcp/
│   │   ├── oauth/
│   │   ├── observability/
│   │   ├── orders/
│   │   ├── provider_compat/
│   │   ├── risk/
│   │   └── sidecar/
│   └── public/
│       ├── audit.rs
│       ├── config.rs
│       ├── gateway.rs
│       ├── mcp.rs
│       ├── orders.rs
│       └── prelude.rs
└── tests/
```

Public consumers should import stable facade modules:

```rust
use ibkr_agent_gateway::prelude::*;
use ibkr_agent_gateway::{Gateway, GatewayConfig};
```

Internal implementation code should remain hidden behind `crate::internal::*`.

## Non-Goals

- Do not publish the internal crates as independent public APIs.
- Do not promise stable internals in `0.1.x`.
- Do not change trading safety semantics during this refactor.
- Do not introduce broker network behavior in packaging tests.

## Phase 1: Public Surface Definition

### Task 1: Define the SDK facade contract

**Description:** Decide the minimum API developers should use when embedding the gateway, independent of the current internal module layout.

**Acceptance criteria:**

- [x] `docs/public-api.md` describes the intended `Gateway`, `GatewayConfig`, backend selection, MCP embedding, audit, and order/risk entrypoints.
- [x] The documented API avoids exposing internal crate names.
- [x] CLI usage and SDK usage are described as two supported entrypoints of the same package.

**Verification:**

- [x] Review imports in docs: no `ibkr_domain`, `ibkr_mcp`, `ibkr_cli`, or other internal crate paths appear as user-facing API.

**Dependencies:** None

**Files likely touched:**

- `docs/public-api.md`
- `README.md`

**Estimated scope:** S

### Task 2: Add root package metadata for crates.io

**Description:** Prepare `Cargo.toml` metadata for a future publish without enabling publish during the early metadata phase.

**Acceptance criteria:**

- [x] Root package has `description`, `readme`, `keywords`, `categories`, `homepage` or `documentation` if useful.
- [x] README includes an "unofficial, not affiliated with Interactive Brokers" disclaimer.
- [x] README explains `cargo install ibkr-agent-gateway` and `cargo add ibkr-agent-gateway` as target workflows.
- [x] `publish = false` remained in place until the migration and release-candidate gates completed.

**Verification:**

- [x] `cargo package --allow-dirty --no-verify --list` shows no `.agents`, `.claude`, `.codex`, `target`, local secrets, or machine-specific files.

**Dependencies:** Task 1

**Files likely touched:**

- `Cargo.toml`
- `README.md`

**Estimated scope:** S

## Phase 2: Root Package Becomes Real Lib + Bin

### Task 3: Move the binary entrypoint to the root package

**Description:** Make `cargo install ibkr-agent-gateway` install the `ibkr-agent` binary from the root package.

**Acceptance criteria:**

- [x] Root package declares or auto-discovers `src/bin/ibkr-agent.rs`.
- [x] `src/bin/ibkr-agent.rs` runs the existing CLI behavior.
- [x] `cargo run --bin ibkr-agent -- health --json` still works.
- [x] `crates/ibkr-cli` is no longer required to install the binary.

**Verification:**

- [x] `cargo run --bin ibkr-agent -- health --json`
- [x] Existing CLI contract tests pass.

**Dependencies:** Task 1

**Files likely touched:**

- `Cargo.toml`
- `src/bin/ibkr-agent.rs`
- `crates/ibkr-cli/src/main.rs`

**Estimated scope:** M

### Task 4: Replace the root test harness with a public library facade

**Description:** Turn `src/lib.rs` into the public library entrypoint instead of a test marker.

**Acceptance criteria:**

- [x] `src/lib.rs` exposes `Gateway`, `GatewayConfig`, `prelude`, and feature-area facade modules.
- [x] Existing top-level integration tests compile against the new root lib.
- [x] Internal modules are not accidentally exposed as public API.

**Verification:**

- [x] `cargo check --workspace`
- [x] `cargo test --workspace`

**Dependencies:** Task 1

**Files likely touched:**

- `src/lib.rs`
- `src/public/*.rs`
- `tests/*`

**Estimated scope:** M

## Phase 3: Migrate Internal Crates Into Root Modules

### Task 5: Migrate leaf crates first

**Description:** Move crates with minimal internal dependencies into `src/internal/*` and update imports.

**Migration order:**

1. `ibkr-domain` -> `src/internal/domain`
2. `ibkr-auth` -> `src/internal/auth`
3. `ibkr-audit` -> `src/internal/audit`
4. `ibkr-cpapi` -> `src/internal/cpapi`
5. `ibkr-risk` -> `src/internal/risk`
6. `ibkr-approval` -> `src/internal/approval`

**Acceptance criteria:**

- [x] Each migrated crate has a `mod.rs` or `lib.rs` equivalent under `src/internal`.
- [x] External-style imports such as `ibkr_domain::...` are replaced with `crate::internal::domain::...`.
- [x] Root package `Cargo.toml` owns required third-party dependencies directly.
- [x] Removed crates are deleted from `[workspace].members`.

**Verification:**

- [x] `cargo check --workspace`
- [x] `cargo test --workspace`

**Implementation note:** Source for `ibkr-domain`, `ibkr-auth`, `ibkr-audit`, `ibkr-cpapi`,
`ibkr-risk`, and `ibkr-approval` now lives under `src/internal/*`. The temporary compatibility
bridge crates have been removed from the workspace and package graph.

**Dependencies:** Task 4

**Files likely touched:**

- `Cargo.toml`
- `src/internal/domain/**`
- `src/internal/auth/**`
- `src/internal/audit/**`
- `src/internal/cpapi/**`
- `src/internal/risk/**`
- `src/internal/approval/**`
- `tests/**`

**Estimated scope:** L, split into one commit per migrated crate.

### Task 6: Migrate service and adapter crates

**Description:** Move higher-level internal crates after their dependencies are available as modules.

**Migration order:**

1. `ibkr-config` -> `src/internal/config`
2. `ibkr-backend` -> `src/internal/backend`
3. `ibkr-sidecar` -> `src/internal/sidecar`
4. `ibkr-oauth` -> `src/internal/oauth`
5. `ibkr-observability` -> `src/internal/observability`
6. `ibkr-orders` -> `src/internal/orders`
7. `ibkr-mcp` -> `src/internal/mcp`
8. `ibkr-provider-compat` -> `src/internal/provider_compat`

**Acceptance criteria:**

- [x] All internal crate imports are converted to module imports.
- [x] No package under `crates/` remains required by the root package.
- [x] `cargo metadata` shows one production package by default.

**Verification:**

- [x] `cargo check --workspace`
- [x] `cargo test --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`

**Implementation note:** Source for `ibkr-config`, `ibkr-backend`, `ibkr-sidecar`,
`ibkr-oauth`, `ibkr-observability`, `ibkr-orders`, `ibkr-mcp`, and
`ibkr-provider-compat` now lives under `src/internal/*`. The root package no longer
depends on any `crates/ibkr-*` path package; integration tests use the root package
and a hidden `ibkr_agent_gateway::testing` facade where they need internal coverage.

**Dependencies:** Task 5

**Files likely touched:**

- `Cargo.toml`
- `src/internal/**`
- `tests/**`

**Estimated scope:** L, split into one commit per migrated crate or dependency layer.

### Task 7: Migrate CLI internals last

**Description:** Move `ibkr-cli` library code into `src/cli` once all service modules are local.

**Acceptance criteria:**

- [x] CLI code imports `crate::internal::*` and public root types where appropriate.
- [x] `crates/ibkr-cli` is removed from the workspace.
- [x] The installed binary remains named `ibkr-agent`.

**Verification:**

- [x] `cargo run --bin ibkr-agent -- health --json`
- [x] `cargo test --workspace`

**Implementation note:** CLI source now lives in `src/cli`, the root binary calls
`ibkr_agent_gateway::cli` directly, and tests import the root package instead of
`ibkr-cli` or the former internal crates.

**Dependencies:** Task 6

**Files likely touched:**

- `src/cli/**`
- `src/bin/ibkr-agent.rs`
- `Cargo.toml`
- `tests/contract_cli_*.rs`

**Estimated scope:** M

## Phase 4: Workspace Cleanup and Publish Readiness

### Task 8: Collapse or remove obsolete workspace members

**Description:** After migration, remove empty `crates/ibkr-*` packages or keep non-published reference directories only if clearly excluded from packaging.

**Acceptance criteria:**

- [x] `[workspace].members` contains only packages intentionally kept for development.
- [x] Root package can build and test without path dependencies to `crates/ibkr-*`.
- [x] `cargo package --list` includes only intended source, tests, docs, config examples, and fixtures.

**Verification:**

- [x] `cargo metadata --format-version 1`
- [x] `cargo package --allow-dirty --no-verify --list`

**Dependencies:** Task 7

**Files likely touched:**

- `Cargo.toml`
- `crates/**`

**Estimated scope:** S

### Task 9: Add package include/exclude rules

**Description:** Make crates.io packaging deterministic and small enough for users.

**Acceptance criteria:**

- [x] Root `Cargo.toml` includes an `include = [...]` or `exclude = [...]` policy.
- [x] Specs and docs included in the crate are intentional.
- [x] Large or local-only artifacts are excluded.
- [x] `.agents`, `.claude`, `.codex`, `.idea`, `.specify`, and `target` are excluded from packages.

**Verification:**

- [x] `cargo package --allow-dirty --no-verify --list`

**Dependencies:** Task 8

**Files likely touched:**

- `Cargo.toml`
- `.gitignore`
- `README.md`

**Estimated scope:** S

### Task 10: Add publish dry-run CI

**Description:** Add a CI check that proves the package can be published before enabling actual release automation.

**Acceptance criteria:**

- [x] CI runs `cargo package --locked --no-verify` or `cargo publish --dry-run --locked`.
- [x] CI runs `cargo install --path . --locked` to verify the binary install path.
- [x] CI runs a smoke command from the installed `ibkr-agent`.

**Verification:**

- [ ] CI passes on a clean checkout.

**Local verification note:** The new package steps were verified locally with
`cargo package --allow-dirty --locked --no-verify`,
`cargo install --path . --locked --force --root /tmp/ibkr-agent-install`,
and `/tmp/ibkr-agent-install/bin/ibkr-agent health --json`.

**Dependencies:** Task 9

**Files likely touched:**

- `.github/workflows/*.yml`

**Estimated scope:** M

## Phase 5: Release Candidate

### Task 11: Stabilize the public API docs and examples

**Description:** Add examples that prove developer embedding works without reaching into internal modules.

**Acceptance criteria:**

- [x] `examples/embed_gateway.rs` uses only `ibkr_agent_gateway::*` public API.
- [x] `examples/run_mcp_stdio.rs` uses the public MCP facade.
- [x] README has a minimal SDK example and CLI install example.

**Verification:**

- [x] `cargo test --examples`
- [x] `cargo run --example embed_gateway`

**Dependencies:** Task 10

**Files likely touched:**

- `examples/*.rs`
- `README.md`
- `docs/public-api.md`

**Estimated scope:** M

### Task 12: Enable publication only after dry-run is clean

**Description:** Remove `publish = false` only once the package is truly publishable.

**Acceptance criteria:**

- [x] `publish = false` is removed from the root package.
- [x] `cargo publish --dry-run --locked` passes.
- [x] `cargo install --path . --locked` installs `ibkr-agent`.
- [x] `cargo audit --deny warnings` passes.

**Verification:**

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `cargo publish --dry-run --locked`

**Dependencies:** Task 11

**Files likely touched:**

- `Cargo.toml`
- `README.md`

**Estimated scope:** S

## Checkpoints

### Checkpoint A: Public Contract

After Tasks 1-2:

- [x] The intended user-facing API is documented.
- [x] The package metadata tells one coherent crates.io story.
- [x] `publish = false` prevented accidental publication during the early refactor phases.

### Checkpoint B: Root Package Works

After Tasks 3-4:

- [x] Root package is a real lib and bin.
- [x] CLI still works.
- [x] Tests still pass.

### Checkpoint C: Internal Crates Migrated

After Tasks 5-7:

- [x] Production code is internal root modules, not path-dependent workspace packages.
- [x] Public API goes through `ibkr_agent_gateway::*`.
- [x] `cargo test --workspace` and clippy pass.

### Checkpoint D: Publish Candidate

After Tasks 8-12:

- [x] Package tarball contents are intentional.
- [x] Install path works.
- [x] SDK examples compile.
- [x] Dry-run publish passes.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Import rewrite churn breaks many tests | High | Migrate one dependency layer at a time and commit each layer separately. |
| Public API accidentally exposes internals | High | Keep internals under `crate::internal` and only re-export through `src/public`. |
| Crates.io package becomes too large | Medium | Use explicit `include`/`exclude` and inspect `cargo package --list`. |
| Existing docs reference internal crate names | Medium | Add a doc grep checkpoint before release. |
| CLI and SDK goals diverge | Medium | Make both adapters use the same `Gateway` facade. |
| One-crate structure makes code harder to navigate | Low | Preserve modular directories under `src/internal/*`. |

## Open Questions

- Should `ibkr-agent-gateway` expose only stable high-level APIs, or also re-export selected domain types for advanced users?
- Should examples be fully offline/fake-backend only for crates.io, or include CPAPI config examples too?
- Decision: `specs/` is kept repository-only and excluded from the crate package.
- Should the first crates.io release be `0.1.0` or a pre-release-style `0.1.0-alpha.1` tag in Git while crates.io still uses `0.1.0`?

## References

- Cargo path dependencies can be used locally, but published packages need registry-resolvable versions for dependencies.
- Cargo manifest metadata should include package description, license, readme, repository, keywords, and categories for crates.io discoverability.
- `cargo package --list` and `cargo publish --dry-run` were the release gates before removing `publish = false`.
