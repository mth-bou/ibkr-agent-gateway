# Paper-to-Live Checklist

Live trading remains disabled until the operator acknowledges this checklist in
configuration and the live request supplies every runtime gate.

For the explicit split between items the gateway enforces mechanically
(config validation, runtime gates) and items the operator must verify in
the deployed environment, see the **"Code-Enforced Gates vs
Operator-Verified Checks"** subsection of
[production-readiness.md](production-readiness.md#live-trading).

Before setting `live_trading.enabled: true` and
`safety.live_trading_enabled: true`:

- run the full paper submit/cancel flow with the same account family
- verify approval records are one-use, unexpired, and account-matched
- configure a live account allowlist with only intended live accounts
- configure a live risk policy id and hard limits for notional, quantity,
  symbol, asset class, frequency, and session exposure
- test the kill switch close/open path
- verify audit storage is writable and live retention is configured
- review the live incident runbook

Live submit requires all of:

- explicit live config and independent safety flag
- allowlisted live account
- live submit scope
- unexpired validated order preview
- matching approval record
- idempotency key
- passing live risk policy
- open kill switch
- audit availability
- acknowledged paper-to-live checklist

If any item is missing, the gateway refuses before broker execution.
