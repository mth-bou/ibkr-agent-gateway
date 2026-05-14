# Read-Only Tools and CLI Commands

This document covers the local read-only MVP surfaces implemented through US2.

## Portfolio and Positions

```bash
ibkr-agent account summary --account DU1234567 --json
ibkr-agent portfolio snapshot --account DU1234567 --json
ibkr-agent positions list --account DU1234567 --json
```

Account-scoped commands require an explicit account id. Missing account context
returns a typed refusal and must not guess between accounts.

## Contracts and Market Data

```bash
ibkr-agent contracts search AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent contracts resolve AAPL --asset-class stock --currency USD --exchange SMART --json
ibkr-agent market snapshot --contract-id 265598 --json
ibkr-agent market bars --contract-id 265598 --duration "1 D" --bar-size "5 mins" --json
```

Supported MVP asset classes are `stock` and `etf`. Ambiguous contract resolution
fails closed. Market data responses include status so delayed, stale, or
unavailable data can be refused or labeled according to policy.

## Read-Only Orders and Executions

```bash
ibkr-agent orders list --account DU1234567 --json
ibkr-agent orders status --account DU1234567 --broker-order-id 123 --json
ibkr-agent executions list --account DU1234567 --json
```

These commands can only inspect existing broker records. Preview, submit,
cancel, modify, and approve commands are refused in this MVP.
