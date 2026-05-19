# Research: MCP Tool Maturity Expansion

## Decision: Implement read-only consultative tools first

**Rationale**: PnL, order history, account metadata, kill switch status, and
limits status improve real usefulness without increasing broker write risk.

**Alternatives considered**:

- Start with modify and stop orders. Rejected because it increases write risk
  before observability and account context are complete.
- Start with options/scanner. Rejected because these are useful but do not fix
  the basic account awareness gap.

## Decision: Keep generic `ibkr_order_modify` forbidden

**Rationale**: Existing tool naming separates paper and live writes. Modify has
the same safety profile as submit/cancel and should be explicit:
`ibkr_paper_order_modify` and `ibkr_live_order_modify`.

**Alternatives considered**:

- Reuse `ibkr_order_modify` with mode input. Rejected because mode-in-payload is
  easier for clients to misuse and weaker than scope-level separation.

## Decision: Add stop and stop-limit before bracket/OCA

**Rationale**: Stop-loss support is a prerequisite for sensible active trading
and is lower complexity than coordinated multi-order groups.

**Alternatives considered**:

- Implement bracket/OCA first. Rejected because it would require group
  lifecycle machinery before the single-order type model is mature.

## Decision: Treat market orders as supported in domain but refused by default

**Rationale**: The model should be complete enough to represent broker order
types, but live market execution is high-risk for agent workflows. Policy should
refuse it unless an operator explicitly enables it.

**Alternatives considered**:

- Remove market orders completely. Rejected because the enum already exists and
  paper/testing flows may need to represent it.

## Decision: Use dedicated scopes for high-impact reads

**Rationale**: Options, depth, scanner, news, fundamentals, transfers, and audit
export have different entitlement, cost, or privacy implications from basic
market data and audit tail.

**Alternatives considered**:

- Reuse broad existing read scopes for all new reads. Rejected because it makes
  deployment policy less precise.

## Decision: Add MCP approval creation only after Phase 1/2 basics

**Rationale**: CLI approval creation already exists. MCP approval creation is
useful, but it changes the MCP workflow write surface and should be added with
clear semantics.

**Alternatives considered**:

- Add it in Phase 1. Rejected because it is not required for consultative use
  and should be reviewed alongside write workflow expansion.
