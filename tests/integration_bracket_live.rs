#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    config::LiveTradingConfig,
    domain::{AccountId, BrokerOrderId, ErrorCode, LocalUserId, OrderGroupId, ValidatedOrderGroup},
    orders::{
        IdempotencyKey, IdempotencyStore, KillSwitch, LiveGroupSubmitRequest,
        LocalCandidateLiveGroupWriter, PaperToLiveMigrationChecklist, submit_live_group_order,
    },
};

#[path = "common/live.rs"]
mod live;

#[tokio::test]
async fn live_bracket_submit_refuses_closed_kill_switch() -> Result<(), Box<dyn std::error::Error>>
{
    let account = AccountId::from_static("U1234567");
    let group = ValidatedOrderGroup {
        group_id: OrderGroupId::new(),
        account_id: account.clone(),
        parent: live::validated_order(account.clone())?,
        take_profit: live::validated_order(account.clone())?,
        stop_loss: live::validated_order(account.clone())?,
    };
    let request = LiveGroupSubmitRequest {
        group,
        idempotency_key: IdempotencyKey::new("live-bracket-closed")?,
        live_config: LiveTradingConfig {
            enabled: true,
            allowed_accounts: vec![account],
            risk_policy_id: Some("test".to_string()),
            paper_to_live_checklist_acknowledged: true,
            reconciler_interval_seconds: 5,
        },
        live_scope_granted: true,
        kill_switch: KillSwitch::closed(LocalUserId::from_static("operator"), "test"),
        audit_available: true,
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    };
    let mut store = IdempotencyStore::default();

    let error = submit_live_group_order(request, &LocalCandidateLiveGroupWriter, &mut store)
        .await
        .err()
        .ok_or("closed kill switch should refuse")?;

    assert_eq!(error.code, ErrorCode::LiveKillSwitchClosed);
    Ok(())
}

#[tokio::test]
async fn live_bracket_submit_returns_three_broker_ids() -> Result<(), Box<dyn std::error::Error>> {
    let account = AccountId::from_static("U1234567");
    let group = ValidatedOrderGroup {
        group_id: OrderGroupId::new(),
        account_id: account.clone(),
        parent: live::validated_order(account.clone())?,
        take_profit: live::validated_order(account.clone())?,
        stop_loss: live::validated_order(account.clone())?,
    };
    let request = LiveGroupSubmitRequest {
        group,
        idempotency_key: IdempotencyKey::new("live-bracket-ok")?,
        live_config: LiveTradingConfig {
            enabled: true,
            allowed_accounts: vec![account],
            risk_policy_id: Some("test".to_string()),
            paper_to_live_checklist_acknowledged: true,
            reconciler_interval_seconds: 5,
        },
        live_scope_granted: true,
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "test"),
        audit_available: true,
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    };
    let mut store = IdempotencyStore::default();

    let lifecycle =
        submit_live_group_order(request, &LocalCandidateLiveGroupWriter, &mut store).await?;
    assert_eq!(lifecycle.broker_order_ids.len(), 3);
    assert!(
        lifecycle
            .broker_order_ids
            .iter()
            .all(|id: &BrokerOrderId| id.as_str().starts_with("live-bracket-"))
    );
    Ok(())
}
