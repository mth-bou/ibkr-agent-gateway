use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_agent_gateway::testing::domain::{AccountId, AccountMarginProfile, AccountMode};

#[tokio::test]
async fn fake_backend_returns_safe_account_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let Some(account_id) = AccountId::new("DU1234567") else {
        return Err("account id rejected".into());
    };

    let metadata = backend.account_metadata(&account_id).await?;

    assert_eq!(metadata.account_id, account_id);
    assert_eq!(metadata.account_mode, AccountMode::Paper);
    assert_eq!(metadata.margin_profile, AccountMarginProfile::Margin);
    assert!(metadata.product_permissions.contains(&"stocks".to_string()));
    assert!(metadata.metadata_redacted);
    Ok(())
}
