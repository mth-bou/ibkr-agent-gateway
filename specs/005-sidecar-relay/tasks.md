# Tasks: Local Sidecar Relay for Retail CP Gateway

## Phase 1: Sidecar Identity and Pairing

- [X] T001 Add `crates/ibkr-sidecar/Cargo.toml` and `crates/ibkr-sidecar/src/lib.rs`.
- [X] T002 Implement sidecar identity in `crates/ibkr-sidecar/src/identity.rs`.
- [X] T003 Implement pairing records in `crates/ibkr-sidecar/src/pairing.rs`.
- [X] T004 Add pairing CLI commands in `crates/ibkr-cli/src/commands/sidecar.rs`.
- [X] T005 Add sidecar config in `crates/ibkr-config/src/sidecar.rs`.

## Phase 2: Relay Transport

- [ ] T010 Implement relay session protocol in `crates/ibkr-sidecar/src/relay.rs`.
- [ ] T011 Implement sidecar heartbeat in `crates/ibkr-sidecar/src/heartbeat.rs`.
- [ ] T012 Implement remote gateway relay endpoint in `crates/ibkr-mcp/src/sidecar_relay.rs`.
- [ ] T013 Implement local CP Gateway forwarding in `crates/ibkr-sidecar/src/client_portal_forwarder.rs`.
- [ ] T014 Implement fail-closed disconnect behavior in `crates/ibkr-sidecar/src/session_state.rs`.

## Phase 3: Tests and Docs

- [ ] T020 Add heartbeat failure tests in `tests/integration_sidecar_heartbeat.rs`.
- [ ] T021 Add session binding tests in `tests/integration_sidecar_pairing.rs`.
- [ ] T022 Add no-secret-forwarding tests in `tests/replay_sidecar_secret_scan.rs`.
- [ ] T023 Add remote/local audit correlation tests in `tests/integration_sidecar_audit.rs`.
- [ ] T024 Document sidecar relay and manual IBKR login boundary in `docs/sidecar-relay.md`.
