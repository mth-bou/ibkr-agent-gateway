#[path = "common/live.rs"]
mod live;

use ibkr_agent_gateway::testing::approval::ApprovalStatus;
use ibkr_agent_gateway::testing::domain::ErrorCode;
use ibkr_agent_gateway::testing::orders::{
    IdempotencyStore, LocalCandidateLiveWriter, submit_live_order,
};
use ibkr_agent_gateway::testing::risk::StaticPolicyRegistry;
use rust_decimal::Decimal;

#[tokio::test]
async fn live_submit_refuses_when_feature_is_disabled() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.live_config.enabled = false;

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let policy_registry = live::live_policy_registry()?;
    let error = submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await;
    let Err(error) = error else {
        return Err("disabled live config must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LiveTradingDisabled);
    assert!(error.message.contains("LIVE_FEATURE_DISABLED"));
    Ok(())
}

#[tokio::test]
async fn live_submit_refuses_missing_live_scope() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.live_scope_granted = false;

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let policy_registry = live::live_policy_registry()?;
    let error = submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await;
    let Err(error) = error else {
        return Err("missing live scope must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LiveGateMissing);
    assert!(error.message.contains("LIVE_SCOPE_MISSING"));
    Ok(())
}

#[tokio::test]
async fn live_submit_refuses_limit_policy_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.live_limit_context.symbol = "MSFT".to_string();

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let policy_registry = live::live_policy_registry()?;
    let error = submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await;
    let Err(error) = error else {
        return Err("symbol limit failure must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LiveLimitRefused);
    assert!(error.message.contains("LIVE_SYMBOL_REFUSED"));
    Ok(())
}

#[tokio::test]
async fn live_submit_refuses_unknown_server_side_policy() -> Result<(), Box<dyn std::error::Error>>
{
    let mut request = live::live_submit_request()?;
    request.live_config.risk_policy_id = Some("missing-policy".to_string());

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let policy_registry = StaticPolicyRegistry::default();
    let error = submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await;
    let Err(error) = error else {
        return Err("unknown live policy id must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LivePolicyUnknown);
    Ok(())
}

#[tokio::test]
async fn live_submit_uses_server_side_policy_limits() -> Result<(), Box<dyn std::error::Error>> {
    let request = live::live_submit_request()?;
    let mut strict_policy = live::live_limit_policy()?;
    strict_policy.max_quantity = Some(ibkr_agent_gateway::testing::domain::Quantity::new(
        Decimal::ZERO,
    ));
    let policy_registry = StaticPolicyRegistry::single(strict_policy);

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let error = submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await;
    let Err(error) = error else {
        return Err("server-side strict policy must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LiveLimitRefused);
    assert!(error.message.contains("LIVE_QUANTITY_LIMIT_REFUSED"));
    Ok(())
}

#[tokio::test]
async fn live_submit_refuses_mismatched_preview_id() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.approval.preview_id = ibkr_agent_gateway::testing::domain::OrderPreviewId::new();

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let policy_registry = live::live_policy_registry()?;
    let error = submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await;
    let Err(error) = error else {
        return Err("mismatched approval preview id must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::ApprovalPreviewMismatch);
    Ok(())
}

#[tokio::test]
async fn live_submit_refuses_consumed_approval() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.approval.status = ApprovalStatus::Consumed;

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let policy_registry = live::live_policy_registry()?;
    let error = submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await;
    let Err(error) = error else {
        return Err("consumed approval must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::ApprovalConsumed);
    Ok(())
}
