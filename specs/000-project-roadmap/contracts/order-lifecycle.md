# Contract: Order Lifecycle

## Preview Path

```text
OrderIntent -> RiskCheckResult -> ValidatedOrder -> OrderPreview
```

Preview does not submit and cannot create broker-side orders.

## Paper Submit Path

```text
OrderPreview -> ApprovalRequest -> ApprovedOrder -> SubmitAttempt -> OrderReceipt -> OrderLifecycleEvent*
```

Paper submit requires:

- paper account mode;
- explicit approval;
- idempotency key;
- unexpired preview;
- audit availability;
- broker session usability.

## Live Submit Path

```text
OrderPreview -> LiveApprovalRequest -> LiveRiskGate -> KillSwitchCheck -> SubmitAttempt -> OrderReceipt
```

Live submit additionally requires:

- live feature enabled;
- account allowlist;
- live scopes;
- live risk limits;
- kill switch open;
- audit retention policy;
- stronger confirmation policy.

## Forbidden State Transitions

- `OrderIntent -> SubmitAttempt`
- `OrderPreview -> SubmitAttempt` without approval
- paper account tools submitting to live accounts
- expired preview submitting
- duplicate idempotency key submitting twice
- live submit while kill switch is closed
