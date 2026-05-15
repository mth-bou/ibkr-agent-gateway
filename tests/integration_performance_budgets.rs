#[path = "common/live.rs"]
mod live;
#[path = "common/remote_oauth.rs"]
mod remote_oauth;

use ibkr_audit::{AuditResultStatus, AuditTailRequest, SqliteAuditWriter};
use ibkr_backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_orders::{IdempotencyKey, IdempotencyStore, submit_live_order};
use ibkr_sidecar::build_forwarded_broker_request;
use std::time::{Duration, Instant};
use time::OffsetDateTime;

#[tokio::test]
async fn fake_backend_read_calls_stay_under_local_budget() -> Result<(), Box<dyn std::error::Error>>
{
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let started = Instant::now();
    let accounts = backend.list_accounts().await?;

    assert!(!accounts.is_empty());
    assert!(started.elapsed() < Duration::from_millis(500));
    Ok(())
}

#[tokio::test]
async fn audit_append_and_tail_stay_under_local_budget() -> Result<(), Box<dyn std::error::Error>> {
    let writer = SqliteAuditWriter::connect("sqlite::memory:").await?;
    let event = ibkr_mcp::build_mcp_tool_event(
        "ibkr_health",
        "ibkr:health:read",
        AuditResultStatus::Completed,
    );

    let append_started = Instant::now();
    writer.append(&event).await?;
    assert!(append_started.elapsed() < Duration::from_millis(50));

    let tail_started = Instant::now();
    let tail = writer.tail(AuditTailRequest::new(10)).await?;
    assert_eq!(tail.events.len(), 1);
    assert!(tail_started.elapsed() < Duration::from_millis(500));
    Ok(())
}

#[test]
fn cached_remote_oauth_validation_stays_under_local_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let token = remote_oauth::rs256_token();
    let config = remote_oauth::oauth_config();
    let jwks = remote_oauth::rsa_jwks();
    let now = OffsetDateTime::now_utc();

    let started = Instant::now();
    for _ in 0..100 {
        let validated = ibkr_oauth::validate_bearer_jwt(
            token,
            &config,
            &jwks,
            Some("ibkr:accounts:read"),
            now,
        )?;
        assert!(validated.granted_scopes.contains("ibkr:accounts:read"));
    }

    assert!(started.elapsed() < Duration::from_millis(50));
    Ok(())
}

#[tokio::test]
async fn audit_tail_over_realistic_local_size_stays_under_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let writer = SqliteAuditWriter::connect("sqlite::memory:").await?;

    for _ in 0..250 {
        let event = ibkr_mcp::build_mcp_tool_event(
            "ibkr_health",
            "ibkr:health:read",
            AuditResultStatus::Completed,
        );
        writer.append(&event).await?;
    }

    let tail_started = Instant::now();
    let tail = writer.tail(AuditTailRequest::new(250)).await?;
    assert_eq!(tail.events.len(), 250);
    assert!(tail_started.elapsed() < Duration::from_millis(500));
    Ok(())
}

#[test]
fn live_gate_risk_and_idempotency_stay_under_order_budget() -> Result<(), Box<dyn std::error::Error>>
{
    let mut idempotency_store = IdempotencyStore::default();
    let started = Instant::now();

    for index in 0..50 {
        let mut request = live::live_submit_request()?;
        request.idempotency_key = IdempotencyKey::new(format!("live-submit-{index}"))?;
        let result = submit_live_order(request, &mut idempotency_store)?;
        assert_eq!(
            result.lifecycle.broker_order_id.as_str(),
            "live-order-local"
        );
    }

    assert!(started.elapsed() < Duration::from_millis(250));
    Ok(())
}

#[test]
fn sidecar_forwarded_request_safety_stays_under_local_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();

    for index in 0..1_000 {
        let forwarded = build_forwarded_broker_request(
            "ibkr_accounts_list",
            "ibkr:accounts:read",
            &serde_json::json!({ "account_hint": format!("paper-{index}") }),
        )?;
        assert_eq!(forwarded.payload_hash.len(), 64);
    }

    assert!(started.elapsed() < Duration::from_millis(100));
    Ok(())
}
