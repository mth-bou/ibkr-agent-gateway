# Contract: Feature Roadmap

| Spec | Name | Must Include | Must Exclude |
|------|------|--------------|--------------|
| `001` | Local Read-Only MVP | CP Gateway read backend, fake backend, CLI, local MCP stdio, local scopes, audit recorder/tail | order preview, submit, cancel, remote MCP, sidecar, live trading |
| `002` | Order Preview + Risk | `OrderIntent`, risk policy, validated order, preview, preview audit | submit, cancel, paper execution, live execution |
| `003` | Paper Submit + Approval | paper submit/cancel, approval, idempotency, lifecycle | live execution, remote OAuth unless already provided by `004` |
| `004` | Remote MCP OAuth/OIDC | HTTP MCP, token validation, scopes, audience/issuer, auth metadata | sidecar relay, new broker write behavior |
| `005` | Sidecar Relay | local sidecar, remote relay, heartbeat, secret isolation | live trading gates unless already provided by `007` |
| `006` | Provider Compatibility | OpenAI/Anthropic/Cursor/Continue/local inspector compatibility | provider-specific broker logic |
| `007` | Live Trading Gated | live account allowlist, limits, kill switch, audit retention, explicit config | live enabled by default |
| `008` | Operations Hardening | observability, audit export, runbooks, packaging | new broker powers |
