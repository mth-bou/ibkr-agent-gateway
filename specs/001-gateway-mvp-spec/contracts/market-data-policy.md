# Contract: Market Data Freshness Policy

## Config

```toml
[market_data]
max_snapshot_age_seconds = 900
allow_delayed = true
stale_policy = "warn" # warn | refuse
```

## Output Requirements

Every market snapshot response MUST include:

```json
{
  "contract_id": "265598",
  "currency": "USD",
  "source_timestamp": "2026-05-14T12:00:00Z",
  "received_at": "2026-05-14T12:00:05Z",
  "data_status": "live",
  "staleness_seconds": 5,
  "warnings": []
}
```

## Data Status

- `live`: source is considered live and fresh.
- `delayed`: broker marks data as delayed, or live entitlement is missing.
- `stale`: source age exceeds configured threshold.
- `unavailable`: broker did not provide usable data.
- `unknown`: adapter cannot prove status.

## Refusal Rules

- If data is unavailable, return `MARKET_DATA_UNAVAILABLE`.
- If data is stale and `stale_policy=refuse`, return `MARKET_DATA_STALE`.
- If data is delayed and `allow_delayed=false`, return `MARKET_DATA_DELAYED`.
- If currency is required but missing, return `MARKET_DATA_INCOMPLETE`.
