---
name: Bug report
about: Report unexpected behavior in ibkr-agent-gateway
title: "bug: "
labels: ["bug", "triage"]
assignees: []
---

<!--
SAFETY NOTICE — before you paste anything:

- Never include broker session cookies, bearer tokens, raw `Authorization`
  headers, IBKR username/password, account numbers (use a redacted form like
  `DU***4567`), API keys, or local file paths that contain credentials.
- The gateway emits redacted audit records by design. Stick to audit output
  rather than raw logs when possible.
- If you suspect a security vulnerability instead of a functional bug, follow
  SECURITY.md instead of opening a public issue.
-->

## Summary

<!-- One sentence describing what is broken. -->

## Expected behavior

<!-- What should have happened. -->

## Actual behavior

<!-- What actually happened. -->

## Steps to reproduce

1.
2.
3.

## Affected surface

<!-- Check all that apply. -->

- [ ] CLI (`ibkr-agent`)
- [ ] Rust SDK (`ibkr_agent_gateway` library)
- [ ] MCP stdio transport
- [ ] MCP HTTP transport (remote)
- [ ] Sidecar relay
- [ ] Audit / SQLite store
- [ ] Other (please describe)

## Environment

- `ibkr-agent-gateway` version (from `Cargo.toml` or `ibkr-agent --version`):
- `rustc` version (`rustc --version`):
- Operating system and architecture:
- Backend used: `fake` / `client-portal` / other
- Client Portal Gateway build (if applicable):

## Redacted logs or output

<!--
Paste short, redacted excerpts inside fenced code blocks.
Replace any sensitive value with `[REDACTED]`.
-->

```text
[REDACTED]
```

## Additional context

<!-- Links to related issues, recent changes, anything else. -->
