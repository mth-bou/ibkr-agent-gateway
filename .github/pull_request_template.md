<!--
Thanks for contributing to ibkr-agent-gateway.

This crate touches broker credentials, order workflows, and audit evidence.
Please read CONTRIBUTING.md before submitting a non-trivial change.
-->

## Summary

<!-- One or two sentences describing the change. Focus on the *why* over the *what*. -->

## Type of change

<!-- Check all that apply. -->

- [ ] `fix` — bug fix
- [ ] `feat` — new feature or capability
- [ ] `refactor` — internal restructuring with no behavior change
- [ ] `perf` — performance improvement
- [ ] `docs` — documentation only
- [ ] `test` — adds or fixes tests
- [ ] `chore` — tooling, CI, release, dependencies
- [ ] `security` — security-affecting change (also see SECURITY.md)

## Test plan

<!--
Describe how a reviewer can verify the change. Include the exact commands you
ran locally. The repo-native gates are:

  cargo fmt --check
  cargo clippy --workspace --all-targets --features unstable-internal-test-support -- -D warnings
  cargo test --workspace --features unstable-internal-test-support
  cargo test --workspace --features unstable-internal-test-support secret
-->

- [ ] `cargo fmt --check`
- [ ] `cargo clippy ... -- -D warnings`
- [ ] `cargo test --workspace --features unstable-internal-test-support`
- [ ] Added or updated tests that exercise the new behavior

## Safety checklist

<!-- All boxes should be either checked or explicitly marked N/A. -->

- [ ] No broker credentials, cookies, bearer tokens, raw headers, or local
      session material are returned to CLI output, MCP responses, logs,
      fixtures, or audit payloads.
- [ ] No live trading enabled by default; new write paths stay behind explicit
      gates (config, scope, approval, idempotency, risk, kill switch, audit,
      paper-to-live checklist where applicable).
- [ ] `CHANGELOG.md` updated under `[Unreleased]` if user-visible behavior or
      the public API surface changes.
- [ ] `docs/` updated when behavior, scopes, error codes, or runbooks change.
- [ ] No `unwrap()` / `expect()` / `panic!` / `todo!` / `dbg!` introduced in
      non-test code.

## Linked issues

<!--
Use `Fixes #123` / `Closes #456` to auto-close on merge. Plain references like
`Refs #789` are fine when the PR only addresses part of an issue.
-->
