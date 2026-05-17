#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::approval::ApprovalService;
use ibkr_agent_gateway::testing::audit::{AuditHmacKey, SqliteAuditWriter};
use ibkr_agent_gateway::testing::domain::{AccountId, ErrorCode, LocalUserId, OrderPreviewId};
use std::sync::Arc;

#[tokio::test]
async fn missing_config_path_is_not_silently_replaced_by_fake_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    let result = ibkr_agent_gateway::cli::run_from_args([
        "ibkr-agent",
        "--config",
        "/tmp/ibkr-agent-gateway-missing-config.yaml",
        "accounts",
        "list",
        "--json",
    ])
    .await;

    let Err(error) = result else {
        return Err("missing config path must fail".into());
    };
    assert_eq!(error.code, ErrorCode::ConfigInvalid);
    Ok(())
}

#[tokio::test]
async fn paper_submit_requires_persisted_approval_and_replays_idempotency()
-> Result<(), Box<dyn std::error::Error>> {
    let key = Arc::new(AuditHmacKey::ephemeral()?);
    let writer = SqliteAuditWriter::connect("sqlite::memory:", key).await?;
    let account = AccountId::from_static("DU1234567");
    let mut approval_service = ApprovalService::default();

    let approval = approval_service.create_approval(
        OrderPreviewId::new(),
        account.clone(),
        LocalUserId::from_static("local-user"),
        300,
    );
    writer.append_approval(&approval).await?;
    let approval_id = approval.approval_id.as_uuid().to_string();

    ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        &approval_id,
        "integration-paper-idempotency-key",
        true,
        true,
    )
    .await?;

    ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        &approval_id,
        "integration-paper-idempotency-key",
        true,
        true,
    )
    .await?;

    let conflicting_approval = approval_service.create_approval(
        OrderPreviewId::new(),
        account.clone(),
        LocalUserId::from_static("local-user"),
        300,
    );
    writer.append_approval(&conflicting_approval).await?;
    let conflict = ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        &conflicting_approval.approval_id.as_uuid().to_string(),
        "integration-paper-idempotency-key",
        true,
        true,
    )
    .await;
    let Err(conflict) = conflict else {
        return Err("same idempotency key with different request must be refused".into());
    };
    assert_eq!(conflict.code, ErrorCode::PaperIdempotencyConflict);

    let missing = ibkr_agent_gateway::cli::commands::orders_paper::submit(
        &writer,
        account.as_str(),
        "019e35d0-aa66-7d33-a057-680d56300b57",
        "integration-paper-missing-approval-key",
        true,
        true,
    )
    .await;
    let Err(missing) = missing else {
        return Err("unknown approval id must be refused".into());
    };
    assert_eq!(missing.code, ErrorCode::PaperApprovalRequired);
    Ok(())
}
