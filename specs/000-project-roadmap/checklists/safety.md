# Safety Requirements Checklist: Complete Roadmap

## Broker Safety

- [x] Read-only phase precedes any order preview.
- [x] Preview/risk phase precedes paper submit.
- [x] Paper submit precedes live submit.
- [x] Live submit is disabled by default and requires multiple independent gates.

## Auth and Audit

- [x] Remote MCP OAuth is modeled as front-door authorization.
- [x] IBKR retail authentication remains manual/local unless future backend modes allow otherwise.
- [x] Audit, redaction, and HMAC account hashing are baseline requirements.

## Provider Neutrality

- [x] OpenAI/Anthropic compatibility is tested through MCP.
- [x] Provider SDKs are forbidden from core broker crates.
