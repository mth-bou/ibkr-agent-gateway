use ibkr_backend::{FakeBackend, FakeFixtureStore, IbkrBackend};
use ibkr_domain::AccountMode;

#[tokio::test]
async fn fake_backend_lists_safe_accounts() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let accounts = backend.list_accounts().await?;

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].account_id.as_str(), "DU1234567");
    assert_eq!(accounts[0].account_mode, AccountMode::Paper);
    assert!(accounts[0].metadata_redacted);
    Ok(())
}
