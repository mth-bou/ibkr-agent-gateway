# Contract: Local CLI Commands

The CLI is an operator and debugging surface for the read-only MVP. Commands
must return human-readable output by default and structured output when the user
requests JSON.

## Commands

```bash
ibkr-agent health
ibkr-agent backend status
ibkr-agent session requirements
ibkr-agent accounts list
ibkr-agent account summary --account U1234567
ibkr-agent positions list --account U1234567
ibkr-agent portfolio snapshot --account U1234567
ibkr-agent contracts search AAPL --asset-class stock --currency USD --exchange SMART
ibkr-agent contracts resolve AAPL --asset-class stock --currency USD --exchange SMART
ibkr-agent market snapshot --contract-id 265598
ibkr-agent market bars --contract-id 265598 --duration "1 D" --bar-size "5 mins"
ibkr-agent orders list --account U1234567
ibkr-agent orders status --account U1234567 --broker-order-id 123
ibkr-agent executions list --account U1234567 --from 2026-05-14T00:00:00Z
ibkr-agent audit tail --limit 100
ibkr-agent mcp serve --transport stdio
```

## Common Flags

- `--config <path>`: load a local gateway config file
- `--json`: output structured JSON
- `--request-id <id>`: optional caller-provided correlation id

## Exit Behavior

- `0`: command completed successfully
- `2`: invalid user input or ambiguous request
- `3`: broker session unavailable or manual action required
- `4`: missing or insufficient scope
- `5`: backend unavailable or returned an unmapped failure
- `6`: unsafe output was detected and refused

## Forbidden Commands

The MVP must not provide commands that submit, cancel, modify, approve, or
preview orders. If a placeholder command exists for discoverability, it must
return an explicit read-only refusal and emit an audit event.
