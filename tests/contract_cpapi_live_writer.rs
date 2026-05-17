//! Contract tests for [`ClientPortalLiveWriter`] against a mocked Client
//! Portal Gateway. Covers the happy path, the reply-chain protocol, the
//! reply-chain depth limit, market-order refusal, missing-limit refusal,
//! and cancel-response parsing.

use ibkr_agent_gateway::testing::cpapi::{ClientPortalClient, ClientPortalLiveWriter};
use ibkr_agent_gateway::testing::domain::{
    AccountId, BrokerOrderId, ContractId, CurrencyCode, ErrorCode, GatewayError, Money,
    OrderIntentId, OrderSide, PreviewOrderType, Quantity, TimeInForce, ValidatedOrder,
    ValidatedOrderId,
};
use ibkr_agent_gateway::testing::orders::{IdempotencyKey, LiveOrderWriter};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use time::{Duration, OffsetDateTime};
use url::Url;
use wiremock::matchers::{body_partial_json, header, method, path};
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
        preview_id: ibkr_agent_gateway::testing::domain::OrderPreviewId::new(),
        intent_id: OrderIntentId::new(),
        account_id: AccountId::from_static("DU1234567"),
        contract_id: ContractId::from_static("265598"),
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
async fn submit_returns_broker_order_id_on_happy_path() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .and(header("content-type", "application/json"))
        .and(body_partial_json(json!({
            "orders": [{
                "acctId": "DU1234567",
                "cOID": "idem-key-1",
                "conid": 265598,
                "orderType": "LMT",
                "side": "BUY",
                "tif": "DAY",
            }]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "order_id": "1234567890",
                "order_status": "Submitted",
                "encrypt_message": "0"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let order = limit_order()?;
    let key = IdempotencyKey::new("idem-key-1")?;
    let receipt = writer.submit_live(&order, &key).await?;
    assert_eq!(receipt.broker_order_id.as_str(), "1234567890");
    assert_eq!(receipt.broker_status.as_deref(), Some("Submitted"));
    Ok(())
}

#[tokio::test]
async fn submit_accepts_numeric_order_id() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "order_id": 42, "order_status": "PreSubmitted" }
        ])))
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let order = limit_order()?;
    let key = IdempotencyKey::new("idem-numeric")?;
    let receipt = writer.submit_live(&order, &key).await?;
    assert_eq!(receipt.broker_order_id.as_str(), "42");
    Ok(())
}

#[tokio::test]
async fn submit_follows_reply_chain_to_confirmation() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": "reply-1",
                "message": ["The price exceeds the daily NAV percentage limit"],
                "isSuppressed": false
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/iserver/reply/reply-1"))
        .and(body_partial_json(json!({ "confirmed": true })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "order_id": "999", "order_status": "Submitted" }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let order = limit_order()?;
    let key = IdempotencyKey::new("idem-reply")?;
    let receipt = writer.submit_live(&order, &key).await?;
    assert_eq!(receipt.broker_order_id.as_str(), "999");
    Ok(())
}

#[tokio::test]
async fn submit_refuses_when_reply_chain_exceeds_limit() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": "reply-a", "message": ["w1"], "isSuppressed": false }
        ])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/iserver/reply/reply-a"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": "reply-b", "message": ["w2"], "isSuppressed": false }
        ])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/iserver/reply/reply-b"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": "reply-c", "message": ["w3"], "isSuppressed": false }
        ])))
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::with_max_reply_rounds(client(&server)?, 1);
    let order = limit_order()?;
    let key = IdempotencyKey::new("idem-loop")?;
    let result = writer.submit_live(&order, &key).await;
    let Err(error) = result else {
        return Err("reply chain depth limit must refuse".into());
    };
    assert_eq!(error.code, ErrorCode::BrokerResponseInvalid);
    Ok(())
}

#[tokio::test]
async fn submit_refuses_market_orders() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let mut order = limit_order()?;
    order.order_type = PreviewOrderType::Market;
    let key = IdempotencyKey::new("idem-market")?;
    let result = writer.submit_live(&order, &key).await;
    let Err(error) = result else {
        return Err("market orders must be refused".into());
    };
    assert_eq!(error.code, ErrorCode::OrderValidationFailed);
    Ok(())
}

#[tokio::test]
async fn submit_refuses_missing_limit_price() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let mut order = limit_order()?;
    order.limit_price = None;
    let key = IdempotencyKey::new("idem-missing-limit")?;
    let result = writer.submit_live(&order, &key).await;
    let Err(error) = result else {
        return Err("missing limit price must be refused".into());
    };
    assert_eq!(error.code, ErrorCode::OrderValidationFailed);
    Ok(())
}

#[tokio::test]
async fn submit_surfaces_session_required_on_401() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let order = limit_order()?;
    let key = IdempotencyKey::new("idem-401")?;
    let result = writer.submit_live(&order, &key).await;
    let Err(error) = result else {
        return Err("401 must be surfaced".into());
    };
    assert_eq!(error.code, ErrorCode::BrokerSessionRequired);
    Ok(())
}

#[tokio::test]
async fn cancel_returns_receipt_for_pending_cancel() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/iserver/account/DU1234567/order/1234567890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "msg": "Request was submitted",
            "order_id": "1234567890",
            "order_status": "PendingCancel",
            "conid": 265598
        })))
        .expect(1)
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let key = IdempotencyKey::new("idem-cancel")?;
    let receipt = writer
        .cancel_live(
            &AccountId::from_static("DU1234567"),
            &BrokerOrderId::from_static("1234567890"),
            &key,
        )
        .await?;
    assert_eq!(receipt.broker_order_id.as_str(), "1234567890");
    assert!(receipt.accepted);
    assert_eq!(receipt.broker_status.as_deref(), Some("PendingCancel"));
    Ok(())
}

#[tokio::test]
async fn cancel_marks_unaccepted_status_as_not_accepted() -> Result<(), Box<dyn std::error::Error>>
{
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/iserver/account/DU1234567/order/777"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "order_id": "777",
            "order_status": "Filled"
        })))
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let key = IdempotencyKey::new("idem-cancel-filled")?;
    let receipt = writer
        .cancel_live(
            &AccountId::from_static("DU1234567"),
            &BrokerOrderId::from_static("777"),
            &key,
        )
        .await?;
    assert!(!receipt.accepted);
    Ok(())
}

#[tokio::test]
async fn submit_serializes_decimal_quantity_and_price_as_numbers()
-> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .and(body_partial_json(json!({
            "orders": [{
                "quantity": 10,
                "price": 123.45
            }]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "order_id": "1", "order_status": "Submitted" }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let order = limit_order()?;
    let key = IdempotencyKey::new("idem-decimal")?;
    let _ = writer.submit_live(&order, &key).await?;
    Ok(())
}

#[tokio::test]
async fn submit_surfaces_broker_error_response_with_explicit_error_field()
-> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/iserver/account/DU1234567/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "error": "Order is not allowed during off-hours" }
        ])))
        .mount(&server)
        .await;

    let writer = ClientPortalLiveWriter::new(client(&server)?);
    let order = limit_order()?;
    let key = IdempotencyKey::new("idem-error")?;
    let result = writer.submit_live(&order, &key).await;
    let Err(error) = result else {
        return Err("broker error field must surface".into());
    };
    assert_eq!(error.code, ErrorCode::BrokerResponseInvalid);
    assert!(error.message.contains("off-hours"));
    let _: Value = json!({});
    Ok(())
}
