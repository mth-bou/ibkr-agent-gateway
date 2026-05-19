# Provider Approval UX

Provider-side approval prompts are optional client UX only. They may help a
human understand what an agent is about to request, but they are not a security
boundary for this gateway.

The gateway must still enforce its own gates before broker access:

- OAuth bearer validation for remote MCP clients
- explicit tool scope checks
- local safety flags for disabled feature classes
- order preview policy and non-executable preview records
- paper order approval records and idempotency checks
- audit events for allowed and refused operations
- redaction of tokens, cookies, credentials, headers, and local paths

A provider approval prompt cannot grant missing gateway scopes, enable disabled
config, authorize paper trading, or permit live trading. If a provider says a
tool call was approved but the gateway policy refuses it, the gateway refusal
wins and returns the stable error code for that gate.

Provider UX should display gateway-visible facts only: tool name, required
scope, user-facing parameters, preview or approval id when one exists, and the
stable refusal or success shape. It must not display or persist broker cookies,
OAuth bearer tokens, Client Portal Gateway session material, raw headers,
credentials, or local filesystem paths.

Any future provider-specific approval integration must remain outside broker
core and should live in compatibility examples or adapters. Broker, risk,
approval, audit, auth, and MCP core semantics stay provider-neutral.

MCP-native `ibkr_approvals_create` creates a gateway approval record, not a
provider UI approval. The tool is scoped with `ibkr:approvals:create`, loads the
preview from server storage, rejects missing, mismatched, or expired previews,
and then persists the approval to the audit database. Provider prompts may
explain that action to a human, but they cannot replace the stored gateway
approval record consumed by paper and live submit flows.
