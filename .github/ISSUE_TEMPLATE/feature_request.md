---
name: Feature request
about: Propose a capability, scope, tool, or workflow addition
title: "feat: "
labels: ["enhancement", "triage"]
assignees: []
---

<!--
This crate intentionally stages broker access behind explicit scopes, typed
requests, deterministic risk gates, idempotency, kill switch, and audit
boundaries. Please describe how your proposal fits inside that model.
-->

## Problem

<!-- What use case are you trying to enable that the gateway does not support today? -->

## Proposed solution

<!--
Describe the change you would like.

Useful structure:
- New MCP tool name(s) and required scope(s)
- New CLI subcommand(s)
- New SDK function(s) or domain type(s)
- Behavior under the existing gates (preview, approval, idempotency, risk,
  kill switch, audit, paper-to-live)
-->

## Alternatives considered

<!-- Workarounds you've tried and why they fall short. -->

## Safety considerations

<!--
Mandatory for any change that touches:
- broker writes (paper, live, modify, cancel, bracket)
- credential, token, or session material
- audit storage
- remote MCP or sidecar relay
- live limits, kill switch, or paper-to-live checklist

Explain how the proposal preserves fail-closed defaults and what error codes
or refusals it should emit when gates are missing.
-->

## Out of scope

<!--
Listing things you are NOT asking for in this issue helps the reviewer keep
scope tight.
-->

## Additional context

<!-- Links to docs/, prior issues, IBKR Client Portal Gateway documentation, etc. -->
