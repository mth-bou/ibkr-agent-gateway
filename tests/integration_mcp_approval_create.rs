#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    audit::{AuditHmacKey, SqliteAuditWriter},
    domain::{AccountId, OrderPreviewId},
};
use std::sync::Arc;

#[tokio::test]
async fn sqlite_persists_mcp_created_approval() -> Result<(), Box<dyn std::error::Error>> {
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let mut service = ibkr_agent_gateway::testing::approval::ApprovalService::default();
    let approval = service.create_approval(
        OrderPreviewId::new(),
        AccountId::from_static("DU1234567"),
        ibkr_agent_gateway::testing::domain::LocalUserId::from_static("mcp-local-user"),
        300,
    );
    writer.append_approval(&approval).await?;

    let loaded = writer
        .load_approval(&approval.approval_id)
        .await?
        .ok_or("approval should be persisted")?;
    assert_eq!(loaded.account_id, AccountId::from_static("DU1234567"));
    Ok(())
}
