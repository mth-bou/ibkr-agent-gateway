#[path = "common/live.rs"]
mod live;

use ibkr_agent_gateway::testing::domain::{ErrorCode, LocalUserId};
use ibkr_agent_gateway::testing::orders::{
    IdempotencyStore, KillSwitch, KillSwitchState, KillSwitchStore, LocalCandidateLiveWriter,
    submit_live_order,
};

#[tokio::test]
async fn closed_kill_switch_refuses_live_submit() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.kill_switch = KillSwitch::closed(LocalUserId::from_static("operator"), "test closed");

    let mut idempotency_store = IdempotencyStore::default();
    let writer = LocalCandidateLiveWriter;
    let policy_registry = live::live_policy_registry()?;
    let error = submit_live_order(request, &writer, &policy_registry, &mut idempotency_store).await;
    let Err(error) = error else {
        return Err("closed kill switch must refuse".into());
    };

    assert_eq!(error.code, ErrorCode::LiveKillSwitchClosed);
    assert!(error.message.contains("LIVE_KILL_SWITCH_CLOSED"));
    Ok(())
}

#[test]
fn emergency_disable_closes_the_kill_switch() {
    let mut store = KillSwitchStore::new_closed(LocalUserId::from_static("operator"), "initial");
    store.open(LocalUserId::from_static("operator"), "ready");

    assert_eq!(store.current().state, KillSwitchState::Open);

    store.emergency_disable(LocalUserId::from_static("operator"), "incident");

    assert_eq!(store.current().state, KillSwitchState::Closed);
}
