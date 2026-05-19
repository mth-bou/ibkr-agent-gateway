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
kill switch refuses live submit, cancel, modify, and bracket submit before
broker execution.

After emergency disable:

- keep audit storage intact
- record the operator, timestamp, reason, request ids, account id hash, and
  affected broker order ids
- stop provider or MCP clients that initiated the flow
- review the last successful preview, approval, submit, cancel, modify, bracket,
  and audit events
- reopen live trading only after limits, scopes, approvals, and audit have been
  verified again

## Live Modify

`ibkr_live_order_modify` is a bounded adjustment path for existing broker
orders. It avoids cancel-and-resubmit races, but it is still a live write: the
handler requires the live modify scope, enabled live config, allowlisted
account, open kill switch, audit availability, and acknowledged paper-to-live
checklist. The MCP payload carries only `account_id`, `broker_order_id`,
`approval_id`, `preview_id`, `idempotency_key`, and bounded changes. The
approval must reference the replacement preview loaded by the server; modify
requests without that approval path fail before the writer boundary.

## Live Bracket

`ibkr_live_bracket_order_submit` is an MCP-only grouped write for parent,
take-profit, and stop-loss legs. It requires approved server-persisted previews
for all three legs, evaluates live limits per leg, inserts durable pending
idempotency state before the writer boundary, and consumes all three approvals
after a successful result. The bundled group writer submits legs sequentially
through the configured `LiveOrderWriter`; it is not broker-native OCA atomicity.

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
