# Live Trading Runbook

Live trading is an operator-controlled mode. It is not enabled by default and
must be reversible at runtime through the kill switch.

## Enablement

1. Complete `docs/paper-to-live.md`.
2. Configure `live_trading.enabled: true`.
3. Add only intended live accounts to `live_trading.allowed_accounts`.
4. Set `live_trading.risk_policy_id` to the deployed live policy.
5. Set `live_trading.paper_to_live_checklist_acknowledged: true`.
6. Set `safety.live_trading_enabled: true`.
7. Confirm audit retention keeps immutable live write events for at least 2555
   days and requires export before purge.

## Emergency Disable

Close the live kill switch immediately when an unexpected order, policy gap,
broker session issue, audit failure, or operator uncertainty appears. A closed
kill switch refuses live submit and cancel before broker execution.

After emergency disable:

- keep audit storage intact
- record the operator, timestamp, reason, request ids, account id hash, and
  affected broker order ids
- stop provider or MCP clients that initiated the flow
- review the last successful preview, approval, submit, cancel, and audit events
- reopen live trading only after limits, scopes, approvals, and audit have been
  verified again

## Incident Review Template

- Incident timestamp:
- Operator:
- Account id hash:
- Tool name:
- Request id:
- Approval id:
- Idempotency key:
- Broker order id:
- Kill switch state before incident:
- Kill switch state after incident:
- Live limit policy id:
- Refusal or execution status:
- Audit event ids:
- Root cause:
- Follow-up changes:
