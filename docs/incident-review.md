# Incident Review

Use this template for refused or unexpected broker-gateway behavior, including
remote MCP auth failures, sidecar relay failures, paper order issues, live gate
refusals, and live kill switch events.

## Immediate Actions

- Close the live kill switch if live trading may be affected.
- Preserve audit storage and avoid destructive cleanup.
- Export relevant audit records as redacted JSONL.
- Record request ids, event ids, approval ids, idempotency keys, and account id
  hashes only.
- Stop any provider, MCP, or sidecar client that is repeating unsafe requests.

## Review Template

- Incident id:
- Time range:
- Summary:
- Affected tools:
- Affected topology:
- Audit event ids:
- Request ids:
- Account id hashes:
- Approval ids:
- Idempotency keys:
- Stable error codes:
- Kill switch state before:
- Kill switch state after:
- Replay fixture path:
- Expected decision:
- Actual decision:
- Root cause:
- Customer/operator impact:
- Follow-up actions:
- Owner:
- Due date:

## Replay Expectations

Replay fixtures must contain redacted audit events and expected structured
decisions only. They must not contain raw bearer tokens, cookies, credentials,
broker session material, local filesystem paths, or live broker dependencies.
