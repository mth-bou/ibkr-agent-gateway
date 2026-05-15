use ibkr_domain::ErrorCode;
use ibkr_sidecar::{
    SidecarHeartbeat, SidecarId, apply_heartbeat, create_relay_session, require_available_session,
};
use time::{Duration, OffsetDateTime};

#[test]
fn heartbeat_updates_bound_session_and_rejects_wrong_sidecar() {
    let sidecar_id = SidecarId::new();
    let mut session = create_relay_session(sidecar_id.clone(), "remote-1", 60);
    let heartbeat = SidecarHeartbeat {
        sidecar_id: sidecar_id.clone(),
        relay_session_id: session.relay_session_id.clone(),
        observed_at: OffsetDateTime::now_utc(),
    };

    apply_heartbeat(&mut session, &heartbeat).expect("matching heartbeat should apply");
    assert!(session.is_available(OffsetDateTime::now_utc()));

    let wrong = SidecarHeartbeat {
        sidecar_id: SidecarId::new(),
        relay_session_id: session.relay_session_id.clone(),
        observed_at: OffsetDateTime::now_utc(),
    };
    let error = apply_heartbeat(&mut session, &wrong).expect_err("wrong sidecar must fail");
    assert_eq!(error.code, ErrorCode::SidecarSessionInvalid);
}

#[test]
fn expired_or_missing_sidecar_session_fails_closed() {
    let sidecar_id = SidecarId::new();
    let session = create_relay_session(sidecar_id, "remote-1", 1);
    let expired_at = OffsetDateTime::now_utc() + Duration::seconds(5);
    let error = require_available_session(Some(&session), expired_at)
        .expect_err("expired sidecar session must fail closed");
    assert_eq!(error.code, ErrorCode::SidecarSessionInvalid);

    let missing = require_available_session(None, OffsetDateTime::now_utc())
        .expect_err("missing sidecar session must fail closed");
    assert_eq!(missing.code, ErrorCode::SidecarUnavailable);
}
