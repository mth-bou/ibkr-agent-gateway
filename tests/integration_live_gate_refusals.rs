#[path = "common/live.rs"]
mod live;

use ibkr_agent_gateway::testing::domain::ErrorCode;
use ibkr_agent_gateway::testing::orders::{IdempotencyStore, submit_live_order};

#[test]
fn live_submit_refuses_when_feature_is_disabled() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.live_config.enabled = false;

    let mut idempotency_store = IdempotencyStore::default();
    let error = submit_live_order(request, &mut idempotency_store);
    let Err(error) = error else {
        return Err("disabled live config must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LiveTradingDisabled);
    assert!(error.message.contains("LIVE_FEATURE_DISABLED"));
    Ok(())
}

#[test]
fn live_submit_refuses_missing_live_scope() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.live_scope_granted = false;

    let mut idempotency_store = IdempotencyStore::default();
    let error = submit_live_order(request, &mut idempotency_store);
    let Err(error) = error else {
        return Err("missing live scope must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LiveGateMissing);
    assert!(error.message.contains("LIVE_SCOPE_MISSING"));
    Ok(())
}

#[test]
fn live_submit_refuses_limit_policy_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.live_limit_context.symbol = "MSFT".to_string();

    let mut idempotency_store = IdempotencyStore::default();
    let error = submit_live_order(request, &mut idempotency_store);
    let Err(error) = error else {
        return Err("symbol limit failure must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LiveLimitRefused);
    assert!(error.message.contains("LIVE_SYMBOL_REFUSED"));
    Ok(())
}
