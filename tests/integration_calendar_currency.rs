#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};

#[tokio::test]
async fn fake_backend_returns_calendar_and_currency() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));

    let session = backend.market_session("NASDAQ").await?;
    assert_eq!(session.exchange, "NASDAQ");
    assert!(session.is_open);

    let holidays = backend.market_holidays("NASDAQ").await?;
    assert_eq!(holidays.holidays[0].name, "Memorial Day");

    let rate = backend.currency_rate("USD", "EUR").await?;
    assert_eq!(rate.base.as_str(), "USD");
    assert_eq!(rate.quote.as_str(), "EUR");
    Ok(())
}
