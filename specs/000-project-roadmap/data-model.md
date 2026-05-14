# Data Model: IBKR Agent Gateway Complete Roadmap

## GatewayInstance

Represents a running local, remote, or sidecar-capable gateway process.

**Fields**

- `instance_id`
- `mode`: `local`, `remote_mcp`, `sidecar`, `remote_with_sidecar`
- `version`
- `started_at`
- `enabled_features`
- `environment`: `dev`, `paper`, `live`

**Validation Rules**

- `live` environment cannot enable live trading unless live gates are configured.
- `remote_mcp` requires OAuth/OIDC authorization.

## AuthContext

Represents the caller identity and authorization source.

**Fields**

- `user_id`
- `source`: `local_config`, `oauth_oidc`, `sidecar_session`
- `issuer`
- `audience`
- `subject`
- `scopes`
- `expires_at`
- `token_id_hash`

**Validation Rules**

- `local_config` has no token expiry/audience semantics in the MVP.
- `oauth_oidc` requires issuer, audience, expiry, and signature validation.
- Tokens are never stored raw.

## BrokerBackend

Represents a concrete broker adapter.

**Fields**

- `backend_type`: `client_portal_gateway`, `ibkr_oauth2_webapi`, `fake`, `tws_optional`
- `base_url`
- `session_status`
- `capabilities`
- `account_modes_supported`

**Validation Rules**

- Backend secrets never leave the backend adapter.
- `client_portal_gateway` is local or sidecar-reached unless explicitly proxied through a secure relay.

## BrokerAccount

Represents a broker account visible to the caller.

**Fields**

- `account_id`
- `account_id_hash`
- `display_label`
- `mode`: `paper`, `live`, `unknown`
- `base_currency`
- `permissions`

**Validation Rules**

- Account mode is required before any write-capable operation.
- Audit defaults to HMAC account hash, not raw account id.

## ContractIdentity

Represents a resolved tradable/readable instrument.

**Fields**

- `contract_id`
- `symbol`
- `asset_class`
- `exchange`
- `currency`
- `description`
- `metadata`

**Validation Rules**

- Ambiguous contract resolution fails closed.
- Unsupported asset classes are explicit refusals.

## MarketDataRecord

Represents market snapshot or historical bars.

**Fields**

- `contract_id`
- `data_kind`: `snapshot`, `historical_bars`
- `bid`
- `ask`
- `last`
- `bars`
- `currency`
- `source_timestamp`
- `received_at`
- `data_status`: `live`, `delayed`, `stale`, `unavailable`
- `staleness_seconds`
- `warnings`

**Validation Rules**

- Currency and timestamps are required.
- Stale/delayed data must be labeled or refused according to policy.

## OrderIntent

Represents a user/agent proposed order. It is not executable.

**Fields**

- `intent_id`
- `account_id`
- `contract_id`
- `side`
- `quantity`
- `order_type`
- `limit_price`
- `time_in_force`
- `rationale`
- `created_by`

**Validation Rules**

- Free-form text may explain rationale but cannot define executable parameters.
- Missing account, contract, side, quantity, or time-in-force fails closed.

## RiskPolicy

Represents deterministic order constraints.

**Fields**

- `policy_id`
- `allowed_account_modes`
- `allowed_asset_classes`
- `max_notional`
- `max_quantity`
- `max_order_count_per_window`
- `concentration_limits`
- `requires_human_approval`
- `live_trading_allowed`

**Validation Rules**

- Live trading defaults to false.
- Policy changes are audited.

## ValidatedOrder

Represents a risk-checked order candidate.

**Fields**

- `validated_order_id`
- `intent_id`
- `account_id`
- `contract_id`
- `broker_order_payload`
- `risk_result`
- `warnings`
- `expires_at`

**Validation Rules**

- Expired validated orders cannot be submitted.
- Validation does not imply approval or submission.

## OrderPreview

Represents a broker/local preview of a validated order.

**Fields**

- `preview_id`
- `validated_order_id`
- `estimated_cost`
- `estimated_commission`
- `margin_impact`
- `warnings`
- `expires_at`

**Validation Rules**

- Preview-only specs cannot submit.
- Preview must bind to a specific validated order.

## ApprovalRequest

Represents explicit permission to submit or cancel.

**Fields**

- `approval_id`
- `preview_id`
- `approved_by`
- `approval_method`: `human_cli`, `human_web`, `policy`
- `decision`: `approved`, `rejected`, `expired`
- `expires_at`

**Validation Rules**

- Approval is single-use.
- Approval expiration prevents submit.

## OrderLifecycleEvent

Represents broker order state.

**Fields**

- `event_id`
- `broker_order_id`
- `account_id_hash`
- `state`: `created`, `submitted`, `pre_submitted`, `filled`, `partially_filled`, `cancelled`, `inactive`, `failed`
- `filled_quantity`
- `remaining_quantity`
- `timestamp`
- `source`

**Validation Rules**

- Unknown broker states are recorded and surfaced as typed warnings.

## SidecarSession

Represents a local sidecar relay connection.

**Fields**

- `sidecar_session_id`
- `local_instance_id`
- `remote_instance_id`
- `user_id`
- `capabilities`
- `heartbeat_at`
- `expires_at`

**Validation Rules**

- Sidecar relay cannot expose local broker cookies, tokens, raw headers, or local filesystem paths.
- Relay disconnect fails closed.

## AuditEvent

Represents append-only redacted evidence.

**Fields**

- `event_id`
- `event_type`
- `timestamp`
- `request_id`
- `correlation_id`
- `user_id`
- `auth_source`
- `account_id_hash`
- `tool_name`
- `scope`
- `decision`
- `result_status`
- `error_code`
- `input_hash`
- `output_hash`
- `redactions`
- `metadata`

**Validation Rules**

- HMAC hash by default for identifiers.
- Raw secrets are never stored.
- Write-capable operations require audit availability.
