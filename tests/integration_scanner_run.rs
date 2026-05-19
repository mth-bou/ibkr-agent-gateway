#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};

#[tokio::test]
async fn fake_backend_runs_scanner() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));

    let scan = backend.scanner_run("TOP_PERC_GAIN").await?;
    assert_eq!(scan.scanner_code, "TOP_PERC_GAIN");
    assert_eq!(scan.results[0].symbol, "AAPL");
    Ok(())
}
