#[path = "common/live.rs"]
mod live;
#[path = "common/remote_oauth.rs"]
mod remote_oauth;

use ibkr_agent_gateway::testing::audit::{
    AuditHmacKey, AuditResultStatus, AuditTailRequest, SqliteAuditWriter,
};
use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_agent_gateway::testing::mcp::http_server::{
    HttpMcpRequest, HttpMcpRuntime, handle_http_mcp_request_with_runtime,
};
use ibkr_agent_gateway::testing::oauth::PreparedOAuthVerifier;
use ibkr_agent_gateway::testing::orders::{
    IdempotencyKey, IdempotencyStore, LocalCandidateLiveWriter, submit_live_order,
};
use ibkr_agent_gateway::testing::sidecar::build_forwarded_broker_request;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use time::OffsetDateTime;

fn test_audit_key() -> Arc<AuditHmacKey> {
    let Ok(key) = AuditHmacKey::ephemeral() else {
        unreachable!("ephemeral key generation must succeed for tests");
    };
    Arc::new(key)
}

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
    let writer = SqliteAuditWriter::connect("sqlite::memory:", test_audit_key()).await?;
    let event = ibkr_agent_gateway::testing::mcp::build_mcp_tool_event(
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
    let verifier = PreparedOAuthVerifier::new(config, &jwks)?;
    let now = OffsetDateTime::now_utc();

    let started = Instant::now();
    for _ in 0..100 {
        let validated = verifier.validate_bearer_jwt(token, Some("ibkr:accounts:read"), now)?;
        assert!(validated.granted_scopes.contains("ibkr:accounts:read"));
    }

    assert!(started.elapsed() < Duration::from_millis(50));
    Ok(())
}

#[test]
fn prepared_remote_mcp_authorization_stays_under_local_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let config = remote_oauth::remote_config()?;
    let jwks = remote_oauth::rsa_jwks();
    let runtime = HttpMcpRuntime::new(&config, &jwks)?;
    let mut headers = BTreeMap::new();
    headers.insert(
        "authorization".to_string(),
        format!("Bearer {}", remote_oauth::rs256_token()),
    );
    let request = HttpMcpRequest {
        path: "/mcp".to_string(),
        headers,
        tool_name: Some("ibkr_accounts_list".to_string()),
        body: serde_json::json!({}),
    };

    let started = Instant::now();
    for _ in 0..100 {
        let response = handle_http_mcp_request_with_runtime(&config, &runtime, &request);
        assert_eq!(response.status, 200);
    }

    assert!(started.elapsed() < Duration::from_millis(50));
    Ok(())
}

#[tokio::test]
async fn audit_tail_over_realistic_local_size_stays_under_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let writer = SqliteAuditWriter::connect("sqlite::memory:", test_audit_key()).await?;

    for _ in 0..250 {
        let event = ibkr_agent_gateway::testing::mcp::build_mcp_tool_event(
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

#[tokio::test]
async fn live_gate_risk_and_idempotency_stay_under_order_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let started = Instant::now();

    for index in 0..50 {
        let mut request = live::live_submit_request()?;
        request.idempotency_key = IdempotencyKey::new(format!("live-submit-{index}"))?;
        let result = submit_live_order(request, &writer, &mut idempotency_store).await?;
        let expected = format!("local-candidate-live-submit-{index}");
        assert_eq!(result.lifecycle.broker_order_id.as_str(), expected);
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
