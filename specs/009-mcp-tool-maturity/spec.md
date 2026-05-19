# Feature Specification: MCP Tool Maturity Expansion

**Feature Branch**: local-only plan, no branch required
**Created**: 2026-05-19
**Status**: Draft for local implementation
**Input**: Gap analysis of the current IBKR MCP tool registry and maturity needs.

## Scope Position in Roadmap

This spec extends the completed read-only, preview, paper, remote OAuth, sidecar,
provider-compatibility, live-gated, and operations-hardening work. It does not
replace the existing roadmap. It is a local implementation plan for increasing
the usable MCP tool surface while preserving the project's explicit, scoped, and
audited safety model.

The current package exposes 20 MCP tools when all local scopes and live gates
are available:

| Category | Tools |
|----------|-------|
| Health | `ibkr_health`, `ibkr_backend_status`, `ibkr_session_requirements` |
| Accounts and portfolio | `ibkr_accounts_list`, `ibkr_account_summary`, `ibkr_positions_list`, `ibkr_portfolio_snapshot` |
| Market data | `ibkr_contracts_search`, `ibkr_contract_resolve`, `ibkr_market_snapshot`, `ibkr_historical_bars` |
| Orders read | `ibkr_orders_list`, `ibkr_order_status`, `ibkr_executions_list` |
| Audit read | `ibkr_audit_tail` |
| Order preview and paper write | `ibkr_order_preview`, `ibkr_paper_order_submit`, `ibkr_paper_order_cancel` |
| Live-gated write | `ibkr_live_order_submit`, `ibkr_live_order_cancel` |

The main missing maturity areas are:

- consultative account awareness: PnL, order history, account permissions and
  restrictions;
- runtime safety visibility: kill switch state, live limits counters, MCP audit
  export, explicit session renewal;
- safer active trading: order modify and stop-style orders;
- advanced research: options chains, greeks, scanner, market depth;
- complex order workflows: bracket and OCA groups;
- contextual data: news, fundamentals, market calendar, currency rates,
  transfers;
- MCP-native approval creation while keeping human approval semantics.

## User Scenarios and Testing

### User Story 1 - Consultative Account Awareness (Priority: P1)

As a user, I want an agent to understand my current and recent account state,
including PnL, past orders, account type, and restrictions, so it can answer
questions and avoid proposing impossible actions.

**Independent Test**: With fake fixtures, MCP calls for PnL, order history, and
account metadata return typed safe payloads, redact account identifiers in audit,
and deny missing scopes.

**Acceptance Scenarios**:

1. **Given** a valid account and portfolio-read scope, **When** the user calls
   `ibkr_pnl_daily`, **Then** the gateway returns realized, unrealized, and total
   daily PnL with currency and timestamp.
2. **Given** a valid account and orders-read scope, **When** the user calls
   `ibkr_orders_history`, **Then** the gateway returns completed, cancelled, and
   expired orders for the requested time range, not only open orders.
3. **Given** a valid account and account-read scope, **When** the user calls
   `ibkr_account_metadata`, **Then** the gateway returns safe metadata such as
   account mode, base currency, margin/cash status, enabled product classes, and
   known restrictions.

### User Story 2 - Runtime Safety Visibility (Priority: P1)

As an operator or agent, I want to know whether trading is disabled and how much
live limit budget remains, so write-capable flows fail clearly before risky
attempts.

**Independent Test**: Kill switch, limits, audit export, and session renewal
tools return safe state without enabling any broker write.

**Acceptance Scenarios**:

1. **Given** live trading is gated, **When** the user calls
   `ibkr_kill_switch_status`, **Then** the gateway returns open/closed state,
   last safe reason, timestamp, and audit correlation.
2. **Given** live limit policies are configured, **When** the user calls
   `ibkr_limits_status`, **Then** the gateway returns order-count and notional
   counters for the active session/window.
3. **Given** audit export scope, **When** the user calls `ibkr_audit_export`,
   **Then** the gateway returns redacted JSONL metadata and payload with the same
   secret scanning guarantees as CLI export.
4. **Given** health-read scope, **When** the user calls `ibkr_session_renew`,
   **Then** the gateway performs keepalive and returns the resulting session
   status without exposing broker session material.

### User Story 3 - Safer Active Trading Adjustments (Priority: P2)

As a user, I want an agent to adjust existing orders and use stop-style
protection without cancel-resubmit races.

**Independent Test**: Paper and live modify tools only accept server-owned order
workflow identifiers, require idempotency, apply approval/risk gates, and never
accept a full executable order payload from MCP.

**Acceptance Scenarios**:

1. **Given** an existing paper order and explicit modify scope, **When** the user
   calls `ibkr_paper_order_modify`, **Then** the gateway loads the existing
   lifecycle state, validates the requested bounded changes, records pending
   idempotency state, calls the writer, and records the modified lifecycle.
2. **Given** an existing live order and explicit live modify scope, **When** the
   user calls `ibkr_live_order_modify`, **Then** live config, scopes, kill switch,
   approval policy, live limits, and audit are checked before the writer
   boundary.
3. **Given** a stop or stop-limit preview, **When** risk checks pass, **Then**
   the preview persists the stop fields and later submit builds the correct
   broker request body.
4. **Given** a generic `ibkr_order_modify` call, **When** the client invokes it,
   **Then** it remains forbidden unless a future spec deliberately replaces the
   explicit paper/live tool split.

### User Story 4 - Advanced Market Research (Priority: P3)

As a user, I want an agent to inspect options, depth, and scanner output so it
can research opportunities before proposing trades.

**Independent Test**: Options chain, greeks, market depth, and scanner tools are
read-only, scope-gated, auditable, and return bounded payloads with clear
freshness and entitlement status.

### User Story 5 - Complex Order Workflows (Priority: P4)

As a user, I want bracket and OCA workflows so entry, take-profit, and stop-loss
orders are treated as one coordinated strategy instead of fragile chained calls.

**Independent Test**: Bracket/OCA preview produces a non-executable group plan;
submit requires approval and writes all group lifecycle records atomically from
the gateway perspective.

### User Story 6 - Contextual Account and Market Data (Priority: P5)

As a user, I want news, fundamentals, market sessions, currency rates, and
transfer history available to the agent for context without granting write
capability.

**Independent Test**: Each contextual tool has a dedicated read scope, safe
output schema, fixture-backed mapping, and redacted audit coverage.

## Requirements

- **FR-001**: The system MUST document the currently exposed MCP tools and keep
  contract tests aligned with the registry.
- **FR-002**: New tools MUST be explicit, domain-specific, scope-filtered, and
  absent from discovery when their scope is missing.
- **FR-003**: Read-only additions MUST not require risk gates but MUST pass audit
  redaction and secret scanning.
- **FR-004**: `ibkr_pnl_daily` MUST expose daily realized, unrealized, total PnL,
  currency, account id hash in audit, and data timestamp.
- **FR-005**: `ibkr_pnl_realtime` MUST expose current session PnL or a structured
  entitlement/session refusal.
- **FR-006**: `ibkr_orders_history` MUST include closed/cancelled/filled orders
  over a bounded requested range and MUST not replace `ibkr_orders_list`.
- **FR-007**: `ibkr_account_metadata` MUST expose safe account permissions and
  restrictions without credentials, tokens, or raw broker session material.
- **FR-008**: `ibkr_kill_switch_status` MUST report current live kill switch
  state without changing it.
- **FR-009**: `ibkr_limits_status` MUST expose live order counters and remaining
  session/window budgets derived from trusted server-side state.
- **FR-010**: `ibkr_audit_export` MUST reuse redacted audit export behavior and
  require a stronger export scope than tail.
- **FR-011**: `ibkr_session_renew` MUST perform keepalive and return safe
  session status.
- **FR-012**: Order modify tools MUST be split by environment:
  `ibkr_paper_order_modify` and `ibkr_live_order_modify`.
- **FR-013**: Generic `ibkr_order_modify` MUST remain forbidden unless another
  accepted spec changes the naming policy.
- **FR-014**: Modify tools MUST use idempotency and write-ahead pending state
  before the broker writer boundary.
- **FR-015**: Modify tools MUST load existing lifecycle/approval/order state from
  server storage; MCP input MUST contain only identifiers and bounded change
  fields.
- **FR-016**: Stop, stop-limit, market, and trailing-stop order types MUST be
  represented explicitly in domain models, risk policy, schemas, fixtures, and
  broker writer body builders.
- **FR-017**: Market orders MUST remain refused by default for live trading until
  explicitly allowed by policy and tests.
- **FR-018**: `ibkr_options_chain` and `ibkr_option_greeks` MUST be read-only and
  must report stale, delayed, unavailable, or missing-entitlement states.
- **FR-019**: `ibkr_market_depth` MUST bound depth levels and return bid/ask book
  rows with venue, price, size, and timestamp when available.
- **FR-020**: `ibkr_scanner_run` MUST use allowlisted scanner codes and bounded
  result sizes.
- **FR-021**: Bracket/OCA workflows MUST have preview-before-submit semantics and
  atomic lifecycle recording from the gateway perspective.
- **FR-022**: News and fundamentals tools MUST treat broker/external text as
  untrusted content and avoid instruction-like rendering in audit.
- **FR-023**: Market session and holiday tools MUST expose whether a target
  instrument or exchange is currently tradable.
- **FR-024**: Currency-rate and transfer-history tools MUST be read-only and
  require explicit scopes.
- **FR-025**: MCP approval creation MUST create an approval record only for an
  existing preview and MUST preserve the distinction between provider UI prompts
  and gateway approval records.
- **FR-026**: Every new tool MUST have registry, schema, scope-denial, handler,
  fake-backend, CPAPI mapping, audit, docs, and contract coverage before being
  considered implemented.

## Non-Goals

- No autonomous live strategy loop.
- No live trading default enablement.
- No broker credential or session export.
- No provider-specific approval semantics inside broker core.
- No raw, unbounded broker payload pass-through for text-heavy endpoints.

## Success Criteria

- **SC-001**: Tool inventory docs and tests agree with the MCP registry for all
  current and added tools.
- **SC-002**: Phase 1 tools allow an agent to answer PnL, order-history, account
  capability, kill-switch, and limit-budget questions without broker writes.
- **SC-003**: Modify and stop-style flows eliminate the need for an agent to
  cancel and recreate orders solely to adjust price/protection.
- **SC-004**: All write-capable additions require explicit scopes, config gates,
  idempotency, audit, and server-loaded state.
- **SC-005**: Advanced research tools are bounded, read-only, entitlement-aware,
  and audit-safe.
- **SC-006**: `cargo fmt --check`, `cargo clippy --workspace --all-targets`, and
  `cargo test --workspace` pass after each implementation phase.

## Key Entities

- **PnlSnapshot**: realized/unrealized/total PnL, currency, account id, period,
  timestamp, data status.
- **HistoricalOrderRecord**: broker order id, account, contract, side, quantity,
  type, prices, status, timestamps, fills, cancellation reason when available.
- **AccountCapabilityProfile**: account mode, base currency, margin/cash status,
  product permissions, shorting/options restrictions, PDT/GFV indicators when
  available.
- **KillSwitchStatus**: current state, changed-by safe id, changed timestamp,
  reason, audit event id.
- **LimitsStatus**: live policy id, session counters, window counters,
  remaining notional, remaining order count, currency.
- **OrderModifyIntent**: existing order id, target account, bounded change set,
  preview/approval ids when required, idempotency key.
- **AdvancedOrderType**: limit, market, stop, stop-limit, trailing-stop with
  explicit required fields.
- **OptionChain**: underlying contract, expiry, strike, right, contract id,
  quote metadata, entitlement status.
- **OptionGreeks**: delta, gamma, theta, vega, implied volatility, model source,
  timestamp.
- **MarketDepthBook**: contract id, bids, asks, venue, depth level, timestamp.
- **ScannerRunResult**: scanner code, filters, bounded rows, snapshot timestamp.
- **BracketOrderGroup**: parent entry, take-profit child, stop-loss child, OCA
  group id, lifecycle state.
