#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::{
    audit::{AuditHmacKey, SqliteAuditWriter},
    auth::{ORDERS_PREVIEW, ScopeSet},
    backend::{FakeBackend, FakeFixtureStore},
    config::LiveTradingConfig,
    domain::LocalUserId,
    mcp::order_groups::{McpOrderGroupContext, handle_bracket_preview},
    orders::{KillSwitch, PaperToLiveMigrationChecklist},
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn mcp_bracket_preview_creates_three_persisted_previews()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let writer =
        SqliteAuditWriter::connect("sqlite::memory:", Arc::new(AuditHmacKey::ephemeral()?)).await?;
    let context = McpOrderGroupContext {
        backend: &backend,
        audit_writer: &writer,
        live_config: LiveTradingConfig::default(),
        kill_switch: KillSwitch::closed(LocalUserId::from_static("operator"), "test"),
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    };
    let scopes = ScopeSet::local_with_preview([ORDERS_PREVIEW])?;

    let payload = handle_bracket_preview(
        &context,
        &scopes,
        &json!({
            "account_id": "DU1234567",
            "symbol": "AAPL",
            "side": "buy",
            "quantity": "1",
            "entry_limit_price": "100",
            "take_profit_limit_price": "110",
            "stop_loss_stop_price": "95"
        }),
    )
    .await?;

    assert!(payload["group_id"].is_string());
    for leg in ["parent", "take_profit", "stop_loss"] {
        let preview_id = payload[leg]["preview_id"]
            .as_str()
            .ok_or("preview id should be serialized")?;
        let record = writer
            .load_order_preview(&ibkr_agent_gateway::testing::domain::OrderPreviewId::parse(
                preview_id,
            )?)
            .await?;
        assert!(record.is_some());
    }
    Ok(())
}
