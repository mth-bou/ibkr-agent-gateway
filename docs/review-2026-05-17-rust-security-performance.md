# Code Review — Rust best practices, Security, Performance

**Date:** 2026-05-17
**Branch:** master
**Reviewed commit:** eb7b8f7 (`perf: optimize gateway hot paths`)
**Scope:** application code under `src/` only — tests, examples, specs, and docs excluded.

## Methodology

Three independent specialist reviews ran in parallel against a fresh read of the codebase, each with its own context:

1. **rust-reviewer** — idiomatic Rust, API ergonomics, error modeling, trait design, async correctness, module organization, dependency hygiene.
2. **security-reviewer** — secret handling, audit log integrity, MCP boundary, HTTP client hygiene, OAuth/JWT validation, sqlx usage, risk gates, crypto.
3. **performance-optimizer** — audit log path, HTTP client setup, lock contention, hot-path allocations, serde patterns, async parallelism, startup cost.

All findings filtered at >70% confidence. Cross-cutting issues identified where multiple reviewers converged.

> **Caveat:** the rust-reviewer was not granted Bash for `cargo check`/`clippy`/`test`; findings are static-analysis based. CI remains the authoritative gate.

## Verdict

Strong structural foundations: zero `unsafe`, workspace lints DENY `unwrap_used`/`expect_used`/`panic`/`todo`/`dbg_macro`, clean `public`/`internal` segmentation, no SQL injection, no command injection. The two CRITICAL findings are isolated design bugs (HMAC placeholder, HS256 acceptance), not systemic. Once corrected, the project sits well above the average open-source Rust hygiene bar.

---

## 🔴 CRITICAL

### C-1 — Account ID HMAC uses a hardcoded placeholder prefix, not a secret key

- **File:** `src/internal/cpapi/mapper.rs:77`
- **Axis:** Security
- **Issue:** `AccountIdHash::new(format!("fixture-hmac:{}", account_id.as_str()))` is invoked in the **production** CPAPI mapper (`map_account`), not only the fake backend. The hash is plain string concatenation — no cryptographic key. An adversary reading the audit log can trivially reverse any `account_id_hash` by brute-forcing the IBKR account ID format (one uppercase letter + seven digits = ~234M candidates, trivial). The design intent in `src/internal/config/mod.rs:42` is `AccountIdMode::Hmac`, but the actual mapper never uses the HMAC primitive from `redaction.rs`.
- **Fix:** Inject a server-side HMAC key from configuration and call `hmac_sha256_hex(key, account_id.as_bytes())` in the mapper. Remove the `"fixture-hmac:"` literal entirely.

### C-2 — HS256 / `oct` JWT tokens are accepted, enabling key-confusion attacks

- **File:** `src/internal/oauth/validator.rs:156–162, 270–271`
- **Axis:** Security
- **Issue:** The verifier parses `"oct"` key types from JWKS and verifies `HS256` JWTs using that symmetric secret. An attacker controlling the JWKS endpoint (or supplying a crafted JWKS URL in config) can present an `"oct"` key whose value matches a known/brute-forceable secret, sign their own JWT with `alg: HS256`, and the validator accepts it. The comment in `jwks.rs:18` acknowledges this is a test artifact.
- **Fix:** Reject `kty=oct` and `alg=HS256` explicitly at the verifier. Accept only `RS256` (extend to `ES256` later via ring's ECDSA) with `kty=RSA`.

---

## 🟠 HIGH

### H-1 — Clock skew overflow silently disables JWT expiry enforcement

- **File:** `src/internal/oauth/validator.rs:332`
- **Axis:** Security
- **Issue:** `i64::try_from(config.clock_skew_seconds).unwrap_or(i64::MAX)` — if an operator configures `clock_skew_seconds` (`u64`) above `i64::MAX`, skew saturates to `i64::MAX` and `claims.exp.saturating_add(i64::MAX)` always exceeds current unix time → every token appears unexpired.
- **Fix:** Cap `clock_skew_seconds` to ≤ 300s in `validate_remote_mcp_config`; hard-error if exceeded.

### H-2 — Fake fixture path not canonicalized → directory traversal via config

- **File:** `src/internal/backend/fake.rs:65`, `src/public/gateway.rs:32`
- **Axis:** Security
- **Issue:** `load_value` uses `self.root.join(relative_path)`. `relative_path` is currently safe (hardcoded), but `fixture_root` (`PathBuf` from `GatewayConfig::fake_with_fixture_root` / `BackendFactoryConfig::fixture_root`) is never canonicalized or restricted. A caller (sidecar relay, test harness) passing `"/"` or `"../../"` resolves to arbitrary filesystem paths.
- **Fix:** Canonicalize at construction time and validate `fixture_root` lies within an expected prefix, or feature-gate the fake backend behind `unstable-internal-test-support`.

### H-3 — No rate limiting on the remote MCP HTTP endpoint

- **File:** `src/internal/mcp/http_server.rs`
- **Axis:** Security
- **Issue:** `handle_http_mcp_request*` performs RSA signature verification on every request. A flood of malformed/expired tokens triggers repeated crypto with no throttling. JWKS cache (10 min) can be kept artificially warm, slowing key rotation.
- **Fix:** Per-IP or per-subject rate limit *before* crypto validation; reject obviously malformed JWTs (missing dot structure) early; bind-layer connection limits.

### H-4 — JWKS HTTP client follows redirects by default → SSRF surface

- **File:** `src/internal/oauth/jwks.rs:59–63`
- **Axis:** Security
- **Issue:** `reqwest::Client::builder()` without explicit policy follows up to 10 redirects. A compromised/misconfigured `jwks_url` or `metadata_url` can pivot via 302 to internal metadata endpoints (`http://169.254.169.254/...`).
- **Fix:** `.redirect(reqwest::redirect::Policy::none())` on the JWKS client; validate `jwks_url` and `metadata_url` are HTTPS before fetching.

### H-5 — Blanket `#[allow(dead_code, unused_imports)]` silences the entire internal hierarchy

- **File:** `src/internal/mod.rs:1–29`
- **Axis:** Rust idiom
- **Issue:** Every internal submodule (100+ files) is silenced. Dead code can accumulate indefinitely without compiler signal. If the suppression exists only for the `unstable-internal-test-support` feature, the allow should be limited to the specific items that need it.
- **Fix:** Remove the blanket allows; delete genuinely unused items or gate them on `#[cfg(feature = "...")]`.

### H-6 — `map_audit_error` discards the underlying `SqlxError` entirely

- **Files:** `src/internal/audit/sqlite.rs:103–110`; same pattern in `backend/fake.rs` and `backend/client_portal.rs` / `cpapi/client.rs`
- **Axis:** Rust idiom (operational impact)
- **Issue:** `map_audit_error(_error: SqlxError)` constructs a generic `AuditWriteFailed` — all diagnostic context lost (SQLITE_FULL, SQLITE_BUSY, constraint violations). Severe production-debugging penalty for a financial audit subsystem.
- **Fix:** Map distinct sqlx variants to distinct `ErrorCode` values; at minimum log `tracing::error!(error = ?_error)` before mapping.

### H-7 — `RwLock` poison permanently disables the fake fixture cache

- **File:** `src/internal/backend/fake.rs:88–101`
- **Axis:** Rust idiom
- **Issue:** Workspace lints prevent `panic`, but third-party panics / OOM can still poison. Because the cache is `Arc<RwLock<…>>` shared via `Clone`, all clones of `FakeFixtureStore` become permanently poisoned, returning `fixture_cache_error()` forever.
- **Fix:** `.unwrap_or_else(|p| p.into_inner())` since the cached JSON invariant is not broken by an unrelated panic, or switch to `parking_lot::RwLock` (no poisoning).

### H-8 — `serde_json::Value` leaks into the public SDK surface

- **File:** `src/internal/backend/trait.rs:8–63`
- **Axis:** Rust idiom (API stability)
- **Issue:** 6 of 13 trait methods return `Result<serde_json::Value>` or `Result<Vec<serde_json::Value>>` with comments "until the portfolio model is introduced in US2." Already reachable via the public `Gateway` type. Shipping `Value` is a breaking-change trap — every caller re-parses, the SDK provides no schema guarantee. (Native async fn in traits is stable since 1.75, but object-safety via `dyn IbkrBackend` still requires `async-trait`.)
- **Fix:** Make these methods `pub(crate)` or feature-gate them until typed models land in US2.

### H-9 — SQLite WAL mode not enabled (highest perf ROI in this review)

- **File:** `src/internal/audit/migrations/0001_audit_events.sql`
- **Axis:** Performance
- **Issue:** Default journal mode (DELETE/rollback) means readers block writers and every commit pays full fsync. With `max_connections(1)` writes are intentionally serialized, but each commit still pays fsync on the main DB file. WAL appends sequentially → 2–5× lower commit latency on most OS/filesystem combos. Single-largest measurable latency win in this review for one line of SQL.
- **Fix:**
  ```sql
  PRAGMA journal_mode = WAL;
  PRAGMA synchronous = NORMAL;
  ```
  `NORMAL` survives OS crash (not hardware power failure) — acceptable for a local audit log.

---

## 🟡 MEDIUM

### Security

- **M-Sec-1 — Audit chain has no tamper-evidence** (`src/internal/audit/sqlite.rs`, `redaction.rs`). `hmac_sha256_hex` and `sha256_hex` are output helpers but no per-event `prev_hash`/`chain_hash` is stored. Rows are independent JSON blobs; deletion/reordering undetectable. *Fix:* add `chain_hash: String` to `AuditEvent` = `HMAC(key, prev_chain_hash || event_id || payload_hash)`; enforce on tail read; alert on gap.
- **M-Sec-2 — `secrecy` / `zeroize` declared workspace-wide but never imported** (`Cargo.toml`, no usage in `src/`). `token_id_hmac_secret` lives in `Vec<u8>` / `String` — recoverable from process dumps / swap. *Fix:* wire into `[dependencies]`, wrap `token_id_hmac_secret` in `secrecy::Secret<Vec<u8>>`, derive `Zeroize` on sensitive structs.
- **M-Sec-3 — Auth error messages echoed verbatim to MCP clients** (`src/internal/mcp/http_auth.rs:133–135`). Strings like `"JWT key id is not present in JWKS"`, `"JWKS key algorithm does not match JWT"` serve as an oracle for algorithm-probing. *Fix:* return a generic `"Authentication failed"` to callers; keep details in server-side logs only.
- **M-Sec-4 — `AuditEvent.metadata` is unredacted at write** (`src/internal/audit/event.rs:131`). Free-form `BTreeMap<String, Value>` serialized as-is into SQLite; the `assert_secret_safe_line` check in `export.rs` runs only at export time and matches a narrow marker list. *Fix:* apply `is_sensitive_field_name` at `AuditEvent` construction; scrub matching values before `SqliteAuditWriter::append`.

### Rust idiom

- **M-Rust-1 — Wrong `ErrorCode` for `direct_broker_oauth_enabled` guard** (`src/internal/config/mod.rs:149–153`). Uses `ConfigRemoteMcpForbidden` for a different concern → callers pattern-matching on code conflate two distinct config failures. *Fix:* new variant or `ConfigInvalid`.
- **M-Rust-2 — Wildcard `_ => 401` in `auth_error_response`** (`src/internal/mcp/http_auth.rs:124–132`). Absorbs non-auth codes (`BrokerBackendUnavailable`, `ConfigInvalid`) as 401. *Fix:* explicit arms for config/broker groups returning 500/503; keep 401 only for auth.
- **M-Rust-3 — `bytes_to_lower_hex` duplicated in 3 files** (`audit/redaction.rs`, `orders/idempotency.rs:184–191`, `oauth/validator.rs:401–408`). *Fix:* extract to `src/internal/encoding.rs`.
- **M-Rust-4 — `resolve_contract` identical in `client_portal.rs:76–99` and `fake.rs:164–188`**. *Fix:* shared `fn resolve_unique(candidates: Vec<ContractCandidate>) -> BackendResult<ContractCandidate>` called from both.
- **M-Rust-5 — `public/config.rs` re-exports 14 internal `validate_*` functions as SDK API** (`src/public/config.rs:1–9`). Couples downstream to internal config signatures. *Fix:* wrap in `GatewayConfiguration::validate()`.
- **M-Rust-6 — `idempotency.rs:145–155` clones a `String` key in `evict_oldest`**. Minor inefficiency at 10k entries.
- **M-Rust-7 — Clippy-eligible `&&[u8]` in `verify_rs256`** (`oauth/validator.rs:292`): `RsaPublicKeyComponents { n: &n, e: &e }` where `n` and `e` are already `&[u8]`. *Fix:* `{ n, e }`.

### Performance

- **M-Perf-1 — `format!("{:?}", event.event_type)` on every audit insert** (`audit/sqlite.rs:49`). Uses `Debug` (`"ToolCalled"`) — inconsistent with the serde snake_case wire form (`"tool_called"`), plus per-call heap alloc. *Fix:* add `AuditEventType::as_str(&self) -> &'static str` returning `&'static` snake_case literals matching serde; `.bind(event.event_type.as_str())`.
- **M-Perf-2 — `is_sensitive_field_name` allocates a `String` per call** (`audit/redaction.rs:40-45`). `to_ascii_lowercase()` heap alloc fires on every field of every audit/log event. *Fix:*
  ```rust
  SENSITIVE_FIELD_NAMES.iter().any(|sensitive| {
      name.len() >= sensitive.len()
          && name.as_bytes().windows(sensitive.len())
              .any(|w| w.eq_ignore_ascii_case(sensitive.as_bytes()))
  })
  ```
- **M-Perf-3 — `broker_tool_schemas()` clones full static `Vec<ToolSchema>` on each call** (`mcp/registry.rs:28-30`). 18 tools × (4 `String` + 2 `Value`) = 36 heap allocs + 18 JSON tree clones per call; `generic_mcp::scenarios()` + `schema_snapshots()` both invoke it. *Fix:* add `broker_tool_schemas_ref() -> &'static [ToolSchema]`; keep owned `Vec` only for mutating callers (`broker_tool_schemas_with_live`).
- **M-Perf-4 — No `[profile.release]` tuning in `Cargo.toml`**. Defaults: `lto = false`, `codegen-units = 16`. *Fix:*
  ```toml
  [profile.release]
  lto = "thin"
  codegen-units = 1
  strip = "debuginfo"
  ```
  Expected: 5–15% reduction in hot-path instruction count for serialization-heavy code (reqwest, sqlx, serde_json).

---

## 🟢 LOW

- **L-1** — `Cargo.toml` declares workspace deps unused by the crate: `tracing`, `wiremock`, `config` (in addition to `secrecy`/`zeroize` flagged above). Remove or move under feature flags.
- **L-2** — `"path"` in `SENSITIVE_FIELD_NAMES` uses `.contains()` → false positives on `dispatch_path`, `update_path` (`audit/redaction.rs:20`). Switch to exact-match set + explicit prefix/suffix matching for headers.
- **L-3** — `GatewayConfig::default()` ⇒ `fake_local()` with relative path `"tests/fixtures/cpapi"` depends on runtime `cwd`. Document or feature-gate `BrokerBackendKind::Fake` behind `unstable-internal-test-support`.
- **L-4** — `GatewayConfiguration` (`config/mod.rs:73–115`): 15 public fields, no builder, no `#[non_exhaustive]` → adding a field breaks downstream construction. Mirror the named-constructor pattern of `GatewayConfig`.
- **L-5** — `bearer_token` lookup is case-sensitive on `BTreeMap<String, String>` for `"Authorization"` (`mcp/http_auth.rs:113–121`). RFC 7230 says header names are case-insensitive — normalize at insertion or use case-insensitive lookup.
- **L-6** — `audit/sqlite.rs:50` stores `event_type` via `format!("{:?}", ...)` (covered above) — also applies to any column where `Debug` representation may drift silently.
- **L-7** — `JwksCache::get_or_fetch` clones the entire `Jwks` on every cache hit (`oauth/jwks.rs:155–161`). *Fix:* store and return `Arc<Jwks>`.
- **L-8** — `FakeFixtureStore::load_json` clones `serde_json::Value` before deserializing (`backend/fake.rs:44`). Test-only path; low priority unless fixtures are loaded under load.
- **L-9** — Pervasive `Some("...".to_string())` hint allocations on `GatewayError`. Migrating `hint` to `Option<Cow<'static, str>>` (or `Option<&'static str>`) eliminates the per-error static-string alloc.

---

## Cross-cutting findings (multiple reviewers converged)

| Finding | Sources | Why it matters |
|---|---|---|
| `secrecy` / `zeroize` declared but unused | Rust L1 + Sec M-2 | Secrets are not actively zeroed despite the dependencies being in `Cargo.toml`. |
| Fixture root path safety | Rust L3 + Sec H-2 | Same relative/uncanonicalized path bites both API stability and security surface. |
| `format!("{:?}", event.event_type)` | Rust L2 + Perf M-Perf-1 | Wire-format inconsistency *and* per-event allocation in the same line. |
| `map_audit_error` drops cause | Rust H-6 | Observability + debuggability in a financial audit subsystem. |

Convergence raises confidence — these should be prioritized.

---

## Recommended action plan

### Sprint 1 — security fixes before any public release

1. **C-1** (real HMAC on account ID) and **C-2** (reject HS256/`oct`) — bloquant pour toute exposition.
2. **H-1**, **H-2**, **H-4** — clock skew overflow, fixture canonicalization, no-redirect JWKS.
3. **H-9** — WAL mode (1 line SQL, highest perf ROI).

### Sprint 2 — Rust hardening

4. **H-5** (remove global `#[allow(dead_code)]`) + **H-6** (preserve sqlx error detail).
5. **H-7** (`RwLock` poison recovery) + **H-8** (`Value` out of public API).
6. **M-Sec-1** (audit chain HMAC) + **M-Sec-4** (redact metadata at write time, not export).

### Sprint 3 — performance and hygiene

7. **M-Perf-1..4** — measurable wins on the audit hot path + release profile.
8. **H-3** (MCP rate limit) — at the time the remote MCP exits the read-only MVP.
9. Rust mediums — DRY (`resolve_contract`, `bytes_to_lower_hex`), builder pattern (`GatewayConfiguration`).

---

## Summary

| Severity | Count |
|---|---|
| CRITICAL | 2 |
| HIGH | 9 |
| MEDIUM | 11 |
| LOW | 9 |
| **Total** | **31** |

**Files implicated (top 10):**

- `src/internal/oauth/validator.rs` (C-2, H-1, M-Rust-7)
- `src/internal/audit/sqlite.rs` (H-6, H-9, M-Perf-1, L-6)
- `src/internal/backend/fake.rs` (H-2, H-7, M-Rust-4, L-8)
- `src/internal/audit/redaction.rs` (M-Sec-4, M-Rust-3, M-Perf-2, L-2)
- `src/internal/mcp/http_auth.rs` (M-Sec-3, M-Rust-2, L-5)
- `src/internal/cpapi/mapper.rs` (C-1)
- `src/internal/oauth/jwks.rs` (H-4, L-7)
- `src/internal/mcp/registry.rs` (M-Perf-3)
- `src/internal/backend/trait.rs` (H-8)
- `src/internal/mod.rs` (H-5)

**Next step:** open one tracking issue per CRITICAL / HIGH, group MEDIUM by axis, batch LOW into a single "polish" issue.
