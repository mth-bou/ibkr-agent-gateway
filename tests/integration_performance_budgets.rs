use ibkr_audit::{AuditResultStatus, AuditTailRequest, SqliteAuditWriter};
use ibkr_backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use std::time::{Duration, Instant};

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
