use ibkr_agent_gateway::testing::domain::ErrorCode;
use ibkr_agent_gateway::testing::orders::{IdempotencyDecision, IdempotencyKey, IdempotencyStore};

#[test]
fn same_idempotency_key_replays_same_request() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = IdempotencyStore::default();
    let key = IdempotencyKey::new("paper-submit-key")?;

    let first = store.record_or_replay(key.clone(), "request-hash")?;
    let second = store.record_or_replay(key, "request-hash")?;

    assert_eq!(first, IdempotencyDecision::New);
    assert_eq!(second, IdempotencyDecision::Replayed);
    Ok(())
}

#[test]
fn same_idempotency_key_rejects_different_request() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = IdempotencyStore::default();
    let key = IdempotencyKey::new("paper-submit-key")?;

    store.record_or_replay(key.clone(), "request-hash")?;
    let Err(error) = store.record_or_replay(key, "different-request-hash") else {
        return Err("different request hash unexpectedly replayed".into());
    };

    assert_eq!(error.code, ErrorCode::PaperIdempotencyConflict);
    Ok(())
}
