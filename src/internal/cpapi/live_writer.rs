//! Client Portal Gateway live writer adapter.
//!
//! Implements [`LiveOrderWriter`] against the Client Portal Web API order
//! placement endpoints. The adapter handles the IBKR reply-chain protocol:
//! `POST /iserver/account/{accountId}/orders` may return either an order id
//! or a list of reply messages requiring confirmation, in which case the
//! adapter posts `POST /iserver/reply/{replyId}` with `{"confirmed": true}`
//! up to a bounded number of rounds and surfaces the final broker-generated
//! order id.

use super::client::ClientPortalClient;
use crate::internal::domain::{
    AccountId, BrokerOrderId, ErrorCode, GatewayError, OrderSide, PreviewOrderType, TimeInForce,
    ValidatedOrder,
};
use crate::internal::orders::{
    IdempotencyKey, LiveCancelReceipt, LiveOrderWriter, LiveSubmitReceipt,
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::{Map, Value};

const DEFAULT_MAX_REPLY_ROUNDS: usize = 5;

/// Live order writer backed by a Client Portal Gateway.
#[derive(Clone)]
pub struct ClientPortalLiveWriter {
    client: ClientPortalClient,
    max_reply_rounds: usize,
}

impl ClientPortalLiveWriter {
    /// Creates a writer with the default reply-chain depth.
    #[must_use]
    pub const fn new(client: ClientPortalClient) -> Self {
        Self {
            client,
            max_reply_rounds: DEFAULT_MAX_REPLY_ROUNDS,
        }
    }

    /// Creates a writer with a custom reply-chain depth.
    #[must_use]
    pub const fn with_max_reply_rounds(
        client: ClientPortalClient,
        max_reply_rounds: usize,
    ) -> Self {
        Self {
            client,
            max_reply_rounds,
        }
    }
}

#[async_trait]
impl LiveOrderWriter for ClientPortalLiveWriter {
    async fn submit_live(
        &self,
        order: &ValidatedOrder,
        idempotency_key: &IdempotencyKey,
    ) -> Result<LiveSubmitReceipt, GatewayError> {
        let body = build_submit_body(order, idempotency_key)?;
        let path = ["iserver", "account", order.account_id.as_str(), "orders"];
        let mut response: Value = self.client.post_json(&path, &body).await?;

        for _ in 0..=self.max_reply_rounds {
            match interpret_submit_response(&response)? {
                SubmitOutcome::Placed {
                    broker_order_id,
                    broker_status,
                } => {
                    return Ok(LiveSubmitReceipt {
                        broker_order_id,
                        broker_status,
                    });
                }
                SubmitOutcome::Reply { reply_id } => {
                    let reply_path = ["iserver", "reply", &reply_id];
                    let confirm = serde_json::json!({ "confirmed": true });
                    response = self.client.post_json(&reply_path, &confirm).await?;
                }
            }
        }
        Err(too_many_replies())
    }

    async fn cancel_live(
        &self,
        account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<LiveCancelReceipt, GatewayError> {
        let path = [
            "iserver",
            "account",
            account_id.as_str(),
            "order",
            broker_order_id.as_str(),
        ];
        let response: Value = self.client.delete_json(&path).await?;
        let object = response.as_object().ok_or_else(invalid_response)?;
        let echoed = read_broker_order_id(object).unwrap_or_else(|| broker_order_id.clone());
        let broker_status = object
            .get("order_status")
            .and_then(Value::as_str)
            .map(str::to_string);
        let accepted = match broker_status.as_deref() {
            Some(status) => is_cancel_accepted_status(status),
            None => object.contains_key("msg"),
        };
        Ok(LiveCancelReceipt {
            broker_order_id: echoed,
            accepted,
            broker_status,
        })
    }
}

pub(super) enum SubmitOutcome {
    Placed {
        broker_order_id: BrokerOrderId,
        broker_status: Option<String>,
    },
    Reply {
        reply_id: String,
    },
}

pub(super) fn interpret_submit_response(response: &Value) -> Result<SubmitOutcome, GatewayError> {
    let array = response.as_array().ok_or_else(invalid_response)?;
    let first = array.first().ok_or_else(empty_response)?;
    let object = first.as_object().ok_or_else(invalid_response)?;
    if let Some(broker_order_id) = read_broker_order_id(object) {
        let broker_status = object
            .get("order_status")
            .and_then(Value::as_str)
            .map(str::to_string);
        return Ok(SubmitOutcome::Placed {
            broker_order_id,
            broker_status,
        });
    }
    if let Some(reply_id) = object.get("id").and_then(Value::as_str) {
        return Ok(SubmitOutcome::Reply {
            reply_id: reply_id.to_string(),
        });
    }
    if let Some(error_message) = object.get("error").and_then(Value::as_str) {
        return Err(GatewayError::new(
            ErrorCode::BrokerResponseInvalid,
            format!("Broker rejected live submit: {error_message}"),
            false,
            Some("Inspect broker response and adjust the order".to_string()),
        ));
    }
    Err(invalid_response())
}

pub(super) fn read_broker_order_id(object: &Map<String, Value>) -> Option<BrokerOrderId> {
    object
        .get("order_id")
        .and_then(|value| match value {
            Value::String(string) => Some(string.clone()),
            Value::Number(number) => Some(number.to_string()),
            _ => None,
        })
        .and_then(BrokerOrderId::new)
}

pub(super) fn is_cancel_accepted_status(status: &str) -> bool {
    let normalized = status.to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "cancelled" | "pendingcancel" | "pending_cancel" | "submitted" | "presubmitted"
    )
}

pub(super) fn build_submit_body(
    order: &ValidatedOrder,
    idempotency_key: &IdempotencyKey,
) -> Result<Value, GatewayError> {
    let order_type = match order.order_type {
        PreviewOrderType::Limit => "LMT",
        PreviewOrderType::Market => {
            return Err(GatewayError::new(
                ErrorCode::OrderValidationFailed,
                "Broker submit refuses market orders",
                false,
                Some("Submit a limit order with an explicit price".to_string()),
            ));
        }
    };
    let side = match order.side {
        OrderSide::Buy => "BUY",
        OrderSide::Sell => "SELL",
    };
    let tif = match order.time_in_force {
        TimeInForce::Day => "DAY",
        TimeInForce::GoodTillCancelled => "GTC",
    };
    let limit_price = order.limit_price.as_ref().ok_or_else(|| {
        GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Broker submit requires a limit price",
            false,
            Some("Provide a limit price in the validated order".to_string()),
        )
    })?;
    let conid: i64 = order
        .contract_id
        .as_str()
        .parse()
        .map_err(|_| invalid_order_field("conid"))?;

    let mut order_object = Map::new();
    order_object.insert(
        "acctId".to_string(),
        Value::String(order.account_id.as_str().to_string()),
    );
    order_object.insert(
        "cOID".to_string(),
        Value::String(idempotency_key.as_str().to_string()),
    );
    order_object.insert("conid".to_string(), Value::Number(conid.into()));
    order_object.insert(
        "orderType".to_string(),
        Value::String(order_type.to_string()),
    );
    order_object.insert("side".to_string(), Value::String(side.to_string()));
    order_object.insert("tif".to_string(), Value::String(tif.to_string()));
    order_object.insert(
        "quantity".to_string(),
        decimal_to_json(order.quantity.value),
    );
    order_object.insert("price".to_string(), decimal_to_json(limit_price.amount));

    Ok(Value::Object({
        let mut map = Map::new();
        map.insert(
            "orders".to_string(),
            Value::Array(vec![Value::Object(order_object)]),
        );
        map
    }))
}

fn decimal_to_json(value: Decimal) -> Value {
    serde_json::from_str::<Value>(&value.to_string())
        .ok()
        .filter(Value::is_number)
        .unwrap_or_else(|| Value::String(value.to_string()))
}

pub(super) fn invalid_response() -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerResponseInvalid,
        "Client Portal Gateway returned an unsupported order response",
        true,
        Some("Inspect broker response safely".to_string()),
    )
}

fn empty_response() -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerResponseInvalid,
        "Client Portal Gateway returned an empty order response",
        true,
        Some("Inspect broker response safely".to_string()),
    )
}

pub(super) fn too_many_replies() -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerResponseInvalid,
        "Client Portal Gateway reply chain exceeded the configured limit",
        false,
        Some("Review broker warnings before resubmitting".to_string()),
    )
}

fn invalid_order_field(field: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::OrderValidationFailed,
        format!("Validated order field is not acceptable for broker submit: {field}"),
        false,
        Some("Use a numeric contract id and a valid order shape".to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::{decimal_to_json, is_cancel_accepted_status};
    use rust_decimal::Decimal;
    use serde_json::Value;

    #[test]
    fn decimal_to_json_emits_numbers_when_possible() {
        assert_eq!(decimal_to_json(Decimal::new(100, 0)), Value::from(100));
        assert_eq!(
            decimal_to_json(Decimal::new(12345, 2)),
            serde_json::from_str::<Value>("123.45").unwrap_or(Value::Null)
        );
    }

    #[test]
    fn cancel_status_accepts_pending_and_cancelled_variants() {
        assert!(is_cancel_accepted_status("Cancelled"));
        assert!(is_cancel_accepted_status("PendingCancel"));
        assert!(is_cancel_accepted_status("Pending_Cancel"));
        assert!(is_cancel_accepted_status("Submitted"));
        assert!(!is_cancel_accepted_status("Filled"));
        assert!(!is_cancel_accepted_status("Rejected"));
    }
}
