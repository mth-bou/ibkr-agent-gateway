//! Contract tests for [`ClientPortalPaperWriter`] against a mocked Client
//! Portal Gateway.

use ibkr_agent_gateway::testing::cpapi::{ClientPortalClient, ClientPortalPaperWriter};
use ibkr_agent_gateway::testing::domain::{
    AccountId, AssetClass, BrokerOrderId, ContractId, CurrencyCode, ErrorCode, GatewayError, Money,
    OrderIntentId, OrderPreviewId, OrderSide, PreviewOrderType, Quantity, TimeInForce,
    ValidatedOrder, ValidatedOrderId,
};
use ibkr_agent_gateway::testing::orders::{IdempotencyKey, PaperOrderWriter};
use rust_decimal::Decimal;
use serde_json::json;
use time::{Duration, OffsetDateTime};
use url::Url;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn limit_order() -> Result<ValidatedOrder, GatewayError> {
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Static currency is invalid",
            false,
            None,
        ));
    };
    Ok(ValidatedOrder {
        validated_order_id: ValidatedOrderId::new(),
        preview_id: OrderPreviewId::new(),
        intent_id: OrderIntentId::new(),
        account_id: AccountId::from_static("DU1234567"),
        contract_id: ContractId::from_static("265598"),
        symbol: Some("AAPL".to_string()),
        asset_class: Some(AssetClass::Stock),
        side: OrderSide::Buy,
        quantity: Quantity::new(Decimal::new(10, 0)),
        order_type: PreviewOrderType::Limit,
        limit_price: Some(Money {
            amount: Decimal::new(12345, 2),
            currency,
        }),
        time_in_force: TimeInForce::Day,
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
        warnings: Vec::new(),
    })
}

fn client(server: &MockServer) -> Result<ClientPortalClient, Box<dyn std::error::Error>> {
    let base = Url::parse(&format!("{}/", server.uri()))?;
    Ok(ClientPortalClient::new(base, false)?)
}

#[tokio::test]
async fn paper_submit_returns_broker_order_id_on_happy_path()
-> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .and(body_partial_json(json!({
            "orders": [{
                "acctId": "DU1234567",
                "cOID": "paper-idem-key-1",
                "conid": 265598,
                "orderType": "LMT",
                "side": "BUY",
                "tif": "DAY",
            }]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "order_id": "paper-1234567890",
                "order_status": "Submitted"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let writer = ClientPortalPaperWriter::new(client(&server)?);
    let order = limit_order()?;
    let key = IdempotencyKey::new("paper-idem-key-1")?;
    let receipt = writer.submit_paper(&order, &key).await?;
    assert_eq!(receipt.broker_order_id.as_str(), "paper-1234567890");
    assert_eq!(receipt.broker_status.as_deref(), Some("Submitted"));
    Ok(())
}

#[tokio::test]
async fn paper_cancel_returns_receipt_for_pending_cancel() -> Result<(), Box<dyn std::error::Error>>
{
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/iserver/account/DU1234567/order/paper-1234567890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "msg": "Request was submitted",
            "order_id": "paper-1234567890",
            "order_status": "PendingCancel"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let writer = ClientPortalPaperWriter::new(client(&server)?);
    let key = IdempotencyKey::new("paper-cancel-key")?;
    let receipt = writer
        .cancel_paper(
            &AccountId::from_static("DU1234567"),
            &BrokerOrderId::from_static("paper-1234567890"),
            &key,
        )
        .await?;
    assert_eq!(receipt.broker_order_id.as_str(), "paper-1234567890");
    assert!(receipt.accepted);
    assert_eq!(receipt.broker_status.as_deref(), Some("PendingCancel"));
    Ok(())
}
