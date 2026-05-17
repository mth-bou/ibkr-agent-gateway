//! Client Portal paper write adapter boundary.
//!
//! Paper trading uses the same Client Portal Gateway order endpoints as live
//! trading, but targets paper accounts. This adapter wires those broker calls
//! behind [`crate::internal::orders::PaperOrderWriter`] so paper validation can
//! run end-to-end before any live workflow is enabled.

use super::client::ClientPortalClient;
use super::live_writer::{
    SubmitOutcome, build_submit_body, interpret_submit_response, is_cancel_accepted_status,
    read_broker_order_id, too_many_replies,
};
use crate::internal::domain::{AccountId, BrokerOrderId, GatewayError, ValidatedOrder};
use crate::internal::orders::{
    IdempotencyKey, PaperCancelReceipt, PaperOrderWriter, PaperSubmitReceipt,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_MAX_REPLY_ROUNDS: usize = 5;

/// Paper submit response mapped from a broker adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPortalPaperSubmitResponse {
    /// Paper broker order id.
    pub broker_order_id: BrokerOrderId,
}

/// Paper cancel response mapped from a broker adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPortalPaperCancelResponse {
    /// Paper broker order id.
    pub broker_order_id: BrokerOrderId,
    /// Whether cancellation was accepted.
    pub accepted: bool,
}

/// Paper order writer backed by a Client Portal Gateway.
#[derive(Clone)]
pub struct ClientPortalPaperWriter {
    client: ClientPortalClient,
    max_reply_rounds: usize,
}

impl ClientPortalPaperWriter {
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
impl PaperOrderWriter for ClientPortalPaperWriter {
    async fn submit_paper(
        &self,
        order: &ValidatedOrder,
        idempotency_key: &IdempotencyKey,
    ) -> Result<PaperSubmitReceipt, GatewayError> {
        let body = build_submit_body(order, idempotency_key)?;
        let path = ["iserver", "account", order.account_id.as_str(), "orders"];
        let mut response: Value = self.client.post_json(&path, &body).await?;

        for _ in 0..=self.max_reply_rounds {
            match interpret_submit_response(&response)? {
                SubmitOutcome::Placed {
                    broker_order_id,
                    broker_status,
                } => {
                    return Ok(PaperSubmitReceipt {
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

    async fn cancel_paper(
        &self,
        account_id: &AccountId,
        broker_order_id: &BrokerOrderId,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<PaperCancelReceipt, GatewayError> {
        let path = [
            "iserver",
            "account",
            account_id.as_str(),
            "order",
            broker_order_id.as_str(),
        ];
        let response: Value = self.client.delete_json(&path).await?;
        let object = response
            .as_object()
            .ok_or_else(super::live_writer::invalid_response)?;
        let echoed = read_broker_order_id(object).unwrap_or_else(|| broker_order_id.clone());
        let broker_status = object
            .get("order_status")
            .and_then(Value::as_str)
            .map(str::to_string);
        let accepted = match broker_status.as_deref() {
            Some(status) => is_cancel_accepted_status(status),
            None => object.contains_key("msg"),
        };
        Ok(PaperCancelReceipt {
            broker_order_id: echoed,
            accepted,
            broker_status,
        })
    }
}
