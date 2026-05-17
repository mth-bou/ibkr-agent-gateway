use ibkr_agent_gateway::testing::domain::ErrorCode;
use ibkr_agent_gateway::testing::sidecar::{
    SidecarHeartbeat, SidecarId, apply_heartbeat, create_relay_session, require_available_session,
};
use time::{Duration, OffsetDateTime};

#[test]
fn heartbeat_updates_bound_session_and_rejects_wrong_sidecar()
-> Result<(), Box<dyn std::error::Error>> {
    let sidecar_id = SidecarId::new();
    let mut session = create_relay_session(sidecar_id.clone(), "remote-1", 60);
    let heartbeat = SidecarHeartbeat {
        sidecar_id: sidecar_id.clone(),
        relay_session_id: session.relay_session_id.clone(),
        observed_at: OffsetDateTime::now_utc(),
    };

    apply_heartbeat(&mut session, &heartbeat)?;
    assert!(session.is_available(OffsetDateTime::now_utc()));

    let wrong = SidecarHeartbeat {
        sidecar_id: SidecarId::new(),
        relay_session_id: session.relay_session_id.clone(),
        observed_at: OffsetDateTime::now_utc(),
    };
    let error = apply_heartbeat(&mut session, &wrong);
    let Err(error) = error else {
        return Err("wrong sidecar must fail".into());
    };
    assert_eq!(error.code, ErrorCode::SidecarSessionInvalid);
    Ok(())
}

#[test]
fn expired_or_missing_sidecar_session_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let sidecar_id = SidecarId::new();
    let session = create_relay_session(sidecar_id, "remote-1", 1);
    let expired_at = OffsetDateTime::now_utc() + Duration::seconds(5);
    let error = require_available_session(Some(&session), expired_at);
    let Err(error) = error else {
        return Err("expired sidecar session must fail closed".into());
    };
    assert_eq!(error.code, ErrorCode::SidecarSessionInvalid);

    let missing = require_available_session(None, OffsetDateTime::now_utc());
    let Err(missing) = missing else {
        return Err("missing sidecar session must fail closed".into());
    };
    assert_eq!(missing.code, ErrorCode::SidecarUnavailable);
    Ok(())
}
