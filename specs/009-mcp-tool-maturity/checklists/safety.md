# Safety Requirements Checklist: MCP Tool Maturity Expansion

- [x] Generic write-like tool names remain forbidden.
- [x] New write tools are environment-specific and explicitly scoped.
- [x] Write tools require server-loaded state, idempotency, pending records, and audit.
- [x] Live write additions preserve config gates, kill switch, approval, risk, and limit checks.
- [x] Read-only additions require audit redaction and safe output handling.
- [x] Audit export uses a stronger scope than audit tail.
- [x] News, fundamentals, scanner labels, and other external text are treated as untrusted.
- [x] No broker credentials, cookies, tokens, or raw session material are exposed.
