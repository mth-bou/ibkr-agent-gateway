#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::backend::{FakeBackend, FakeFixtureStore, IbkrBackend};

#[tokio::test]
async fn fake_backend_returns_news_and_fundamentals() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));

    let news = backend.news_list("AAPL").await?;
    assert_eq!(news.symbol, "AAPL");
    assert_eq!(news.articles[0].article_id, "news-1");

    let article = backend.news_article("news-1").await?;
    assert_eq!(article.article_id, "news-1");

    let fundamentals = backend.fundamentals_get("AAPL").await?;
    assert_eq!(fundamentals.symbol, "AAPL");
    assert!(fundamentals.fields.get("pe_ratio").is_some());
    Ok(())
}
