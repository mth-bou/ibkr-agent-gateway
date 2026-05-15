#[path = "common/live.rs"]
mod live;

use ibkr_domain::{ErrorCode, LocalUserId};
use ibkr_orders::{KillSwitch, KillSwitchState, KillSwitchStore, submit_live_order};

#[test]
fn closed_kill_switch_refuses_live_submit() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = live::live_submit_request()?;
    request.kill_switch = KillSwitch::closed(LocalUserId::from_static("operator"), "test closed");

    let error = submit_live_order(request).expect_err("closed kill switch must refuse");

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
