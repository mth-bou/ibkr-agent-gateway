# Security Policy

`ibkr-agent-gateway` mediates access to Interactive Brokers accounts. A
vulnerability in this crate can leak broker session material, bypass order
gates, or corrupt audit evidence, so security reports are taken seriously even
on a small project.

This project is not affiliated with, endorsed by, or supported by Interactive
Brokers. Vulnerabilities in the Interactive Brokers Client Portal Gateway
itself should be reported to Interactive Brokers, not here.

## Supported Versions

Only the most recent minor release line on `master` is supported with security
fixes. Older lines may receive backports at the maintainer's discretion when
the fix is mechanical.

| Version | Supported |
|---------|-----------|
| 0.5.x   | ✅        |
| < 0.5.0 | ❌        |

Patch releases (`0.5.y`) are published whenever a security fix lands.

## Reporting a Vulnerability

**Do not** open a public GitHub issue, PR, discussion, or social media post
that describes the vulnerability before a fix is released.

Use one of these private channels:

1. **GitHub Private Vulnerability Reporting** (preferred):
   <https://github.com/mth-bou/ibkr-agent-gateway/security/advisories/new>
2. **Email** the maintainer at <mathieu.boucher55@gmail.com> with the subject
   line `ibkr-agent-gateway security report`.

Please include:

- A description of the issue and its impact in the gateway's threat model.
- A minimal reproduction (commands, fixtures, or test case). The
  `unstable-internal-test-support` feature is the easiest harness for
  reproducible PoCs.
- The affected version(s), backend (`fake` / `client-portal`), and transport
  (CLI / MCP stdio / MCP HTTP / sidecar relay).
- Whether the reporter wants public credit after disclosure.

Do not include real broker credentials, real account numbers, or real session
material in the report. Replace them with placeholders before sending.

## Response Process

- Acknowledgment within **3 business days** of receipt.
- Triage and severity assessment within **7 business days**.
- A coordinated fix and release are planned with the reporter. Disclosure
  timelines are negotiated case by case; the default target is **90 days**
  from acknowledgment, shortened for critical issues with active exploitation.
- A GitHub Security Advisory is published alongside the patched release. The
  CHANGELOG entry is added under the `### Security` section for that release.

## Scope

The following are explicitly in scope and treated as high-severity by default:

- Broker credential, cookie, bearer token, raw header, OAuth secret, account
  number, or local session material leaking to CLI output, MCP responses,
  logs, fixtures, or audit payloads.
- Bypass of any documented order gate: preview enforcement, approval
  consumption, idempotency, risk policy, allowlist, kill switch, audit
  availability, paper-to-live checklist, live limit policy.
- Tampering with the audit chain (HMAC chain hash, redaction trail,
  account-id hashing) or producing audit events that can't be verified.
- OAuth/OIDC validation flaws on the remote MCP HTTP path: algorithm
  confusion, signature confusion, issuer/audience/scope bypass, JWKS
  selection ambiguity.
- Sidecar relay forwarding paths that accept payloads they should refuse.
- Concurrency / race conditions that allow approvals or idempotency keys to
  be reused, orders to be duplicated, or `pending_writer` records to be lost.

The following are out of scope unless they cause one of the above:

- Bugs in the Interactive Brokers Client Portal Gateway itself.
- Operator-side misconfiguration (running with permissive policy, real broker
  credentials in plaintext fixtures, etc.) when the gateway already refused
  the unsafe state at startup.
- Denial of service through legitimate API surface (rate limit exhaustion is
  a feature, not a bug, unless it bypasses authentication or the kill switch).
- Issues that require attacker-controlled physical access to the host running
  `ibkr-agent`.

## Hardening Expectations

Operators are expected to:

- Run the local Client Portal Gateway over localhost only; the config
  validator enforces this.
- Store `CARGO_REGISTRY_TOKEN`, OAuth credentials, JWKS material, and audit
  HMAC keys outside the repo (`.env`, secret store, OS keychain).
- Treat `unstable-internal-test-support` as an internal feature; never enable
  it in production builds.
- Watch the `ibkr-agent audit verify` chain output and quarantine the host on
  first verification failure.

For non-security questions, use issues or discussions on the public repo
instead.
