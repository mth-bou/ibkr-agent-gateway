# Contributing to ibkr-agent-gateway

Thanks for considering a contribution. This crate is small but it sits between
agents and broker order workflows, so contributions are reviewed against a
narrow set of rules:

- the safety model documented in [README.md](README.md) and
  [docs/production-readiness.md](docs/production-readiness.md) is
  non-negotiable;
- defaults are fail-closed; new features must keep that property;
- every change runs through the same gates as the maintainer's own work.

If you only need to file a security report, follow [SECURITY.md](SECURITY.md)
instead.

## Project Layout

The crate is a single Cargo project with two entry points (`ibkr-agent` CLI
and `ibkr_agent_gateway` library). Code is organized by domain, not by file
type. A high-level walkthrough is in [docs/developer-guide.md](docs/developer-guide.md).
Specifications under `specs/` drive feature work and explain why a given
boundary exists.

## Dev Environment

Requirements:

- Rust toolchain declared in `rust-toolchain.toml` (currently `1.94.1` stable
  with `rustfmt` and `clippy`).
- SQLite available on the host (sqlx-sqlite uses the bundled driver, no
  separate install needed).
- Optional: `cargo-audit` for local dependency advisory checks.

Smoke-test from a fresh checkout:

```bash
cargo run --bin ibkr-agent -- health --json
cargo run --bin ibkr-agent -- accounts list --json
cargo run --bin ibkr-agent -- mcp serve --transport stdio --describe --json
```

These commands hit the offline fake backend under `tests/fixtures/cpapi`, so
no real broker session is required.

## Quality Gates

Every change must pass the same gates that CI runs. Run them locally before
opening a PR:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
cargo test --workspace --features unstable-internal-test-support
cargo test --workspace --features unstable-internal-test-support secret
```

Useful packaging checks before releases:

```bash
cargo package --allow-dirty --no-verify --list
cargo publish --dry-run --locked
```

The workspace lints set `unsafe_code = forbid` and treat
`clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::todo`,
and `clippy::dbg_macro` as `deny`. Tests are the only place where `unwrap()`
and `expect()` are acceptable, and even there an `else { unreachable!() }`
pattern is preferred when the failure mode would be confusing.

## Testing Expectations

See [docs/testing.md](docs/testing.md) for the canonical reference. In short:

- New broker write paths require integration tests that exercise the gates
  (preview, approval, idempotency, risk, kill switch, audit, paper-to-live
  where applicable).
- New CPAPI client paths require wiremock or fake-fixture contract tests so
  the gateway boundary is locked.
- New MCP tools require schema, scope-filter, and audit tests.
- Security-sensitive changes should add a regression test under
  `tests/replay_*.rs` or `tests/contract_*.rs` next to the existing ones.
- Concurrency-sensitive changes need a `tokio::join!`-style regression test;
  see `tests/integration_audit_sqlite.rs::concurrent_live_workflow_completion_keeps_transaction_connection`
  for the pattern.

## Commits

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): summary in present tense, lowercase

Optional body explaining the *why*. Wrap at 80 columns. Reference issues
or PRs with `Fixes #123` / `Refs #456` at the end of the body.
```

Types used in this repo: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`,
`chore`, `ci`, `security`.

Common scopes: `orders`, `mcp`, `cpapi`, `audit`, `oauth`, `risk`, `sidecar`,
`config`, `cli`, `release`, `spec`.

Do **not** add LLM or AI attribution lines to commits (e.g.,
`Co-Authored-By: <some LLM>`). The maintainer's `git log` is human-only.

## Pull Requests

1. Fork and create a feature branch off `master`.
2. Make the change in the smallest reasonable diff. Don't bundle unrelated
   refactors.
3. Update `CHANGELOG.md` under `[Unreleased]` for any user-visible behavior,
   public API surface, or scope change.
4. Update `docs/` whenever behavior, scopes, error codes, or runbooks change.
5. Make sure the quality gates above are green locally.
6. Open the PR using the template (`Description`, `Type of change`,
   `Test plan`, `Safety checklist`, `Linked issues`).
7. The maintainer reviews against the safety model first, then the
   architecture, then the diff itself.

CI runs `ci.yml` (fmt + clippy + test + docs + package smoke) and
`security.yml` (cargo audit + secret regression). Both must pass.

## Release Workflow

Maintainer-only. Recorded here so contributors know how to verify a
release without inventing tags.

1. Make sure `[Unreleased]` in `CHANGELOG.md` accurately reflects what is
   about to ship.
2. Bump `version` in `Cargo.toml`. Refresh `Cargo.lock` with `cargo check`.
3. Convert `[Unreleased]` to `[X.Y.Z] - YYYY-MM-DD` and add the
   `compare/vA...vB` reference line.
4. Commit as `chore(release): vX.Y.Z`.
5. Annotated tag: `git tag -a vX.Y.Z -m "vX.Y.Z"`.
6. `cargo publish --dry-run --locked` to verify the package.
7. Push the branch then the tag: `git push origin master && git push origin vX.Y.Z`.
8. The `release.yml` workflow handles `cargo publish --locked` and the
   GitHub Release. Tags created before `release.yml` was added are published
   manually with `cargo publish --locked`.

The tag ruleset on the GitHub repository forbids force-update and deletion of
release tags. Hotfixes always cut a new patch version.

## Documentation

Operator-facing docs live under `docs/`. The developer-facing source of truth
is `README.md` plus inline rustdoc on the public SDK surface. Spec-level
records go under `specs/` with a stable numeric prefix; never edit an old
spec in place once it has shipped — fork a new one.

## License

By contributing you agree that your contribution is licensed under the MIT
License of this project (see [LICENSE](LICENSE)). You confirm that you have
the right to license the code you submit.
