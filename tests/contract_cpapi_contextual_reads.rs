//! Contract tests for contextual Client Portal Gateway read endpoints.
//!
//! These tests lock the concrete HTTP paths and query parameters used by
//! `ClientPortalClient` for Spec 009 market/contextual reads. Fake-backend
//! integration tests cover domain mapping; these contracts cover the CPAPI
//! boundary itself.

#![cfg(feature = "unstable-internal-test-support")]

use ibkr_agent_gateway::testing::cpapi::ClientPortalClient;
use serde_json::{Value, json};
use url::Url;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Result<ClientPortalClient, Box<dyn std::error::Error>> {
    let base = Url::parse(&format!("{}/", server.uri()))?;
    Ok(ClientPortalClient::new(base, false)?)
}

#[tokio::test]
async fn options_chain_uses_secdef_options_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/secdef/options"))
        .and(query_param("symbol", "AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "symbol": "AAPL",
            "expirations": ["2026-06-19"]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.options_chain("AAPL").await?;

    assert_eq!(payload["symbol"], "AAPL");
    Ok(())
}

#[tokio::test]
async fn option_greeks_use_market_snapshot_fields() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/marketdata/snapshot"))
        .and(query_param("conids", "700001"))
        .and(query_param("fields", "delta,gamma,theta,vega,iv"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "conid": "700001",
                "delta": "0.52",
                "gamma": "0.03",
                "theta": "-0.01",
                "vega": "0.12",
                "iv": "0.24"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.option_greeks("700001").await?;

    assert_eq!(payload[0]["conid"], "700001");
    Ok(())
}

#[tokio::test]
async fn market_depth_uses_depth_endpoint() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/marketdata/depth"))
        .and(query_param("conid", "265598"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "conid": "265598",
            "bids": [],
            "asks": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.market_depth("265598").await?;

    assert_eq!(payload["conid"], "265598");
    Ok(())
}

#[tokio::test]
async fn scanner_run_uses_scanner_code_query() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/scanner/run"))
        .and(query_param("scanner", "TOP_PERC_GAIN"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "scanner": "TOP_PERC_GAIN",
            "contracts": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.scanner_run("TOP_PERC_GAIN").await?;

    assert_eq!(payload["scanner"], "TOP_PERC_GAIN");
    Ok(())
}

#[tokio::test]
async fn news_list_uses_symbol_query() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/news/list"))
        .and(query_param("symbol", "AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "articles": [{"article_id": "article-1"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.news_list("AAPL").await?;

    assert_eq!(payload["articles"][0]["article_id"], "article-1");
    Ok(())
}

#[tokio::test]
async fn news_article_uses_article_id_query() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/news/article"))
        .and(query_param("article_id", "article-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "article_id": "article-1",
            "body": "redacted fixture body"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.news_article("article-1").await?;

    assert_eq!(payload["article_id"], "article-1");
    Ok(())
}

#[tokio::test]
async fn fundamentals_get_uses_symbol_query() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/fundamentals"))
        .and(query_param("symbol", "AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "symbol": "AAPL",
            "ratios": {}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.fundamentals_get("AAPL").await?;

    assert_eq!(payload["symbol"], "AAPL");
    Ok(())
}

#[tokio::test]
async fn market_calendar_reads_use_exchange_query() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/marketdata/session"))
        .and(query_param("exchange", "NASDAQ"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "exchange": "NASDAQ",
            "is_open": true
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/iserver/marketdata/holidays"))
        .and(query_param("exchange", "NYSE"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "exchange": "NYSE",
            "holidays": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let cpapi = client(&server)?;
    let session = cpapi.market_session("NASDAQ").await?;
    let holidays = cpapi.market_holidays("NYSE").await?;

    assert_eq!(session["exchange"], "NASDAQ");
    assert_eq!(holidays["exchange"], "NYSE");
    Ok(())
}

#[tokio::test]
async fn currency_rate_uses_base_and_quote_queries() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/currency/rate"))
        .and(query_param("base", "USD"))
        .and(query_param("quote", "EUR"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "base": "USD",
            "quote": "EUR",
            "rate": "0.92"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.currency_rate("USD", "EUR").await?;

    assert_eq!(payload["quote"], "EUR");
    Ok(())
}

#[tokio::test]
async fn transfer_history_uses_portfolio_account_path() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/portfolio/DU1234567/transfers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "transfers": [{"transfer_id": "transfer-1"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload = client(&server)?.transfer_history("DU1234567").await?;

    assert_eq!(payload["transfers"][0]["transfer_id"], "transfer-1");
    Ok(())
}

#[tokio::test]
async fn contextual_read_query_values_are_encoded_not_interpolated()
-> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/iserver/news/list"))
        .and(query_param("symbol", "BRK B"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "articles": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let payload: Value = client(&server)?.news_list("BRK B").await?;

    assert_eq!(payload["articles"], json!([]));
    Ok(())
}
