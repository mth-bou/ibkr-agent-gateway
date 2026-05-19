use ibkr_agent_gateway::testing::audit::{AuditHmacKey, AuditTailRequest, SqliteAuditWriter};
use ibkr_agent_gateway::testing::auth::{AUDIT_EXPORT, AUDIT_READ, ScopeSet};
use ibkr_agent_gateway::testing::mcp::local_tool_schemas_for_scopes;
use std::sync::Arc;

#[tokio::test]
async fn audit_export_scope_is_distinct_from_tail() -> Result<(), Box<dyn std::error::Error>> {
    let read_scopes = ScopeSet::read_only([AUDIT_READ])?;
    let read_tools = local_tool_schemas_for_scopes(&read_scopes)
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();
    assert!(read_tools.contains(&"ibkr_audit_tail".to_string()));
    assert!(!read_tools.contains(&"ibkr_audit_export".to_string()));

    let export_scopes = ScopeSet::read_only([AUDIT_EXPORT])?;
    let export_tools = local_tool_schemas_for_scopes(&export_scopes)
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();
    assert!(export_tools.contains(&"ibkr_audit_export".to_string()));
    assert!(!export_tools.contains(&"ibkr_audit_tail".to_string()));

    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let export = writer.export_jsonl(AuditTailRequest::new(10)).await?;
    assert_eq!(export.range.exported_count, 0);
    assert!(export.payload_jsonl.is_empty());
    Ok(())
}
