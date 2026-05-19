# Implementation Plan: MCP Tool Maturity Expansion

**Branch**: local-only | **Date**: 2026-05-19 | **Spec**: [spec.md](./spec.md)

## Summary

Increase the IBKR MCP tool surface from the current analyst/paper/live-gated
baseline to a more mature gateway. The implementation should be staged so the
lowest-risk read-only tools land first, then write-capable order improvements,
then advanced research and complex order groups.

The core rule is unchanged: every broker-facing tool is explicit, scoped,
audited, bounded, and backed by typed domain models. Write tools must use
server-loaded state, idempotency, pending write-ahead records, and the existing
preview to approval to submit pattern.

## Baseline Tool Inventory

Baseline registry source:

- `src/internal/mcp/registry.rs`
- `src/internal/mcp/tools/*.rs`
- `src/cli/commands/mcp.rs`
- `src/internal/backend/trait.rs`

Pre-009 MCP surface with all local scopes:

| Tool | Scope | Status |
|------|-------|--------|
| `ibkr_health` | `ibkr:health:read` | Present |
| `ibkr_backend_status` | `ibkr:health:read` | Present |
| `ibkr_session_requirements` | `ibkr:health:read` | Present |
| `ibkr_accounts_list` | `ibkr:accounts:read` | Present |
| `ibkr_account_summary` | `ibkr:portfolio:read` | Present |
| `ibkr_positions_list` | `ibkr:positions:read` | Present |
| `ibkr_portfolio_snapshot` | `ibkr:portfolio:read` | Present |
| `ibkr_contracts_search` | `ibkr:marketdata:read` | Present |
| `ibkr_contract_resolve` | `ibkr:marketdata:read` | Present |
| `ibkr_market_snapshot` | `ibkr:marketdata:read` | Present |
| `ibkr_historical_bars` | `ibkr:marketdata:read` | Present |
| `ibkr_orders_list` | `ibkr:orders:read` | Present, open/read-only orders only |
| `ibkr_order_status` | `ibkr:orders:read` | Present |
| `ibkr_executions_list` | `ibkr:orders:read` | Present |
| `ibkr_audit_tail` | `ibkr:audit:read` | Present |
| `ibkr_order_preview` | `ibkr:orders:preview` | Present, limit-only MCP input |
| `ibkr_paper_order_submit` | `ibkr:orders:paper:submit` | Present |
| `ibkr_paper_order_cancel` | `ibkr:orders:paper:cancel` | Present |
| `ibkr_live_order_submit` | `ibkr:orders:live:submit` | Present, live-gated |
| `ibkr_live_order_cancel` | `ibkr:orders:live:cancel` | Present, live-gated |

Main maturity verdict:

- The pre-009 package was strong for `LLM-as-analyst` and controlled paper/live
  single-order workflows.
- It was not mature enough for `LLM-as-active-trader` because modify,
  stop-style orders, PnL visibility, and historical order analysis were missing
  from MCP.
- The highest-value additions were mostly read-only and low-risk.

## Technical Context

**Language/Version**: Rust workspace, Rust 2024 edition.

**Primary Runtime Paths**:

- MCP registry: `src/internal/mcp/registry.rs`
- MCP schema helpers: `src/internal/mcp/schemas.rs`
- MCP stdio dispatch: `src/cli/commands/mcp.rs`
- MCP write handlers: `src/internal/mcp/order_workflows.rs`,
  `src/internal/mcp/live_orders.rs`
- Backend trait: `src/internal/backend/trait.rs`
- Fake backend: `src/internal/backend/fake.rs`
- Client Portal backend: `src/internal/backend/client_portal.rs`
- CPAPI client/mapper/models: `src/internal/cpapi/client.rs`,
  `src/internal/cpapi/mapper.rs`, `src/internal/cpapi/models.rs`
- Order domain: `src/internal/domain/order_preview.rs`,
  `src/internal/domain/order.rs`
- Risk: `src/internal/risk/checks.rs`, `src/internal/risk/live_limits.rs`
- Writers: `src/internal/orders/live_writer.rs`,
  `src/internal/cpapi/live_writer.rs`
- Audit and idempotency: `src/internal/audit/sqlite.rs`,
  `src/internal/orders/pending.rs`
- Scope constants: `src/internal/auth/scopes.rs`
- Public exports: `src/public/*.rs`

**Testing**:

- MCP registry/schema snapshots.
- Scope denial tests.
- Fake backend integration tests.
- CPAPI mapper and writer contract tests.
- Audit redaction and export tests.
- Idempotency and pending write recovery tests for write tools.
- Performance budget tests for high-frequency read tools.

## Constitution Check

- **Provider-neutral MCP**: PASS. All additions remain MCP tools and CLI/backend
  primitives, not provider-specific SDK calls.
- **Deterministic finance core**: PASS if every response is typed and every
  order action uses deterministic validation.
- **Read-only first**: PASS. Phase 1 is read-only and safety visibility.
- **Separated auth**: PASS. New scopes are gateway scopes, not broker auth.
- **Audit first**: PASS if each new tool records called/completed/failed/denied
  events before release.
- **Risk before submit**: PASS if modify/bracket/stops still flow through
  preview, approval, risk, and live gates.
- **Paper before live**: PASS. Implement paper modify/bracket before live where
  possible.

## Phase Plan

### Phase 0 - Registry and Contract Baseline

Purpose: lock current behavior before expanding it.

Deliverables:

- Tool inventory contract in `specs/009-mcp-tool-maturity/contracts/mcp-tools.md`.
- Tests that current tool list remains scope-filtered.
- Docs confirming generic write names remain forbidden.

Implementation notes:

- Do not expose `ibkr_order_modify` as a generic tool.
- Keep all new write tool names environment-specific.

### Phase 1 - Consultative and Safety Read Tools

Purpose: make the gateway usable for real advisory workflows before adding more
write capability.

Tools:

- `ibkr_pnl_daily`
- `ibkr_pnl_realtime`
- `ibkr_orders_history`
- `ibkr_account_metadata`
- `ibkr_kill_switch_status`
- `ibkr_limits_status`
- `ibkr_audit_export`
- `ibkr_session_renew`

Scopes:

- Reuse `ibkr:portfolio:read` for PnL.
- Reuse `ibkr:orders:read` for order history.
- Reuse `ibkr:accounts:read` for account metadata.
- Reuse `ibkr:health:read` for session renew and kill-switch status.
- Add `ibkr:risk:read` visibility to `LOCAL_SCOPES` for limits status if not
  already discoverable as a tool scope.
- Add `ibkr:audit:export` for MCP export because export is higher-impact than
  tail.

Risk:

- Low. These are read-only or operational visibility tools.
- Main risk is accidental leakage in audit/export payloads.

### Phase 2 - Order Modify and Protective Order Types

Purpose: remove cancel-resubmit pressure and add basic stop-loss capability.

Tools:

- `ibkr_paper_order_modify`
- `ibkr_live_order_modify`

Domain additions:

- `PreviewOrderType::Stop`
- `PreviewOrderType::StopLimit`
- `PreviewOrderType::TrailingStop`
- Keep `PreviewOrderType::Market`, but live policy refuses it by default.
- Add stop price, trailing amount/percent, and optional limit price validation.

Implementation pattern:

- Add modify writer methods rather than overloading submit/cancel.
- Require idempotency keys.
- Insert pending state before writer call.
- Load existing lifecycle and preview/approval state from SQLite.
- For live modify, run live config, scope, kill switch, market snapshot, and
  limit checks before writer boundary.

Risk:

- Medium. This is write-capable.
- Must add paper modify first, then live modify.

### Phase 3 - Advanced Market Research

Purpose: enable options and intraday research without write capability.

Tools:

- `ibkr_options_chain`
- `ibkr_option_greeks`
- `ibkr_market_depth`
- `ibkr_scanner_run`

Scopes:

- Add `ibkr:options:read`.
- Add `ibkr:marketdata:depth:read` or reuse `ibkr:marketdata:read` with a
  stricter result-size limit. Prefer a new depth scope if entitlement or cost is
  meaningfully different.
- Add `ibkr:scanner:read`.

Risk:

- Medium. Payload sizes and entitlements must be controlled.
- Treat all descriptions and external text as untrusted.

### Phase 4 - Bracket Workflows

Purpose: represent multi-leg risk-managed strategies as one coordinated group.

Tools:

- `ibkr_bracket_order_preview`
- `ibkr_paper_bracket_order_submit`
- `ibkr_live_bracket_order_submit`

Implementation pattern:

- Preview creates a non-executable group plan.
- Approval binds to each server-persisted bracket leg preview.
- Submit writes a pending group transaction before any writer call.
- Broker partial failure must produce recoverable lifecycle state.
- OCA is future scope unless a dedicated writer and contract tests are added.

Risk:

- High. This should wait until Phase 2 is stable.

### Phase 5 - Contextual Read Tools

Purpose: add broker-provided context useful for decision support.

Tools:

- `ibkr_news_list`
- `ibkr_news_article`
- `ibkr_fundamentals_get`
- `ibkr_market_holidays`
- `ibkr_market_session`
- `ibkr_currency_rate`
- `ibkr_transfer_history`
- `ibkr_approvals_create`

Notes:

- `ibkr_approvals_create` is not a broker write, but it is a gateway workflow
  write. Gate it with `ibkr:approvals:create` and require an existing preview.
- News and fundamentals should have strict output size and redaction behavior.
- Market calendar/session should be prioritized before fundamentals if active
  trading support is the goal.

## Target Tool Backlog

| Priority | Tool | Primary scope | Rationale |
|----------|------|---------------|-----------|
| P1 | `ibkr_pnl_daily` | `ibkr:portfolio:read` | Basic performance visibility |
| P1 | `ibkr_pnl_realtime` | `ibkr:portfolio:read` | Current session awareness |
| P1 | `ibkr_orders_history` | `ibkr:orders:read` | Retro analysis |
| P1 | `ibkr_account_metadata` | `ibkr:accounts:read` | Avoid impossible recommendations |
| P1 | `ibkr_kill_switch_status` | `ibkr:health:read` | Runtime safety clarity |
| P1 | `ibkr_limits_status` | `ibkr:risk:read` | Remaining live budget |
| P1 | `ibkr_audit_export` | `ibkr:audit:export` | MCP parity with CLI export |
| P1 | `ibkr_session_renew` | `ibkr:health:read` | MCP parity with keepalive intent |
| P2 | `ibkr_paper_order_modify` | `ibkr:orders:paper:modify` | Avoid cancel-resubmit in paper |
| P2 | `ibkr_live_order_modify` | `ibkr:orders:live:modify` | Avoid cancel-resubmit in live |
| P2 | stop/stop-limit/trailing support | existing preview/write scopes | Protective order support |
| P3 | `ibkr_options_chain` | `ibkr:options:read` | Options research |
| P3 | `ibkr_option_greeks` | `ibkr:options:read` | Options risk |
| P3 | `ibkr_market_depth` | `ibkr:marketdata:depth:read` | Intraday/depth research |
| P3 | `ibkr_scanner_run` | `ibkr:scanner:read` | Idea generation |
| P4 | bracket tools | paper/live group scopes | Coordinated strategy lifecycle |
| P5 | news/fundamentals/calendar/currency/transfers | dedicated read scopes | Context enrichment |
| P5 | `ibkr_approvals_create` | `ibkr:approvals:create` | MCP-native approval record creation |

## Verification Gates

Run after each phase:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace --features unstable-internal-test-support
```

Additional gates for write phases:

```bash
cargo test --workspace --features unstable-internal-test-support --test integration_mcp_live_submit
cargo test --workspace --features unstable-internal-test-support --test integration_live_idempotency
cargo test --workspace --features unstable-internal-test-support --test integration_live_kill_switch
```

Additional gates for registry/schema changes:

```bash
cargo test --workspace --features unstable-internal-test-support --test contract_mcp_broker_tool_list
cargo test --workspace --features unstable-internal-test-support --test contract_mcp_schemas
```
