//! Typed non-executable order preview models.

use super::account::AccountMode;
use super::contract::AssetClass;
use super::identifiers::{AccountId, AuditEventId, ContractId, LocalUserId};
use super::money::{CurrencyCode, Money, Quantity};
use super::order::OrderSide;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Order intent identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct OrderIntentId(Uuid);

/// Validated order identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ValidatedOrderId(Uuid);

/// Order preview identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct OrderPreviewId(Uuid);

macro_rules! impl_preview_uuid_id {
    ($type_name:ident) => {
        impl $type_name {
            /// Creates a fresh UUID-backed identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Returns the inner UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }

            /// Parses an identifier from a UUID string.
            pub fn parse(value: &str) -> Result<Self, super::error::GatewayError> {
                let uuid = Uuid::parse_str(value).map_err(|_| {
                    super::error::GatewayError::new(
                        super::error::ErrorCode::OrderValidationFailed,
                        concat!(stringify!($type_name), " must be a valid UUID"),
                        false,
                        Some("Use an identifier returned by the gateway".to_string()),
                    )
                })?;
                Ok(Self(uuid))
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

impl_preview_uuid_id!(OrderIntentId);
impl_preview_uuid_id!(ValidatedOrderId);
impl_preview_uuid_id!(OrderPreviewId);

/// Contract input for preview validation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OrderContractInput {
    /// Use an already resolved broker contract id.
    ContractId {
        /// Broker contract id.
        contract_id: ContractId,
    },
    /// Resolve from explicit instrument context.
    Query {
        /// Symbol or query.
        symbol: String,
        /// Asset class.
        asset_class: AssetClass,
        /// Currency.
        currency: CurrencyCode,
        /// Exchange or route.
        exchange: Option<String>,
    },
}

/// Preview order type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PreviewOrderType {
    /// Limit order candidate.
    Limit,
    /// Market order candidate, expected to be refused by default risk policy.
    Market,
}

/// Time in force for preview-only order candidates.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    /// Day order.
    Day,
    /// Good-till-cancelled candidate.
    GoodTillCancelled,
}

/// Non-executable order proposal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OrderIntent {
    /// Intent id.
    pub intent_id: OrderIntentId,
    /// Account id.
    pub account_id: AccountId,
    /// Account mode expected by policy.
    pub account_mode: AccountMode,
    /// Contract input.
    pub contract: OrderContractInput,
    /// Buy or sell.
    pub side: OrderSide,
    /// Quantity.
    pub quantity: Quantity,
    /// Order type.
    pub order_type: PreviewOrderType,
    /// Limit price, required for limit order candidates.
    pub limit_price: Option<Money>,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Optional safe explanatory text. This never defines executable fields.
    pub rationale: Option<String>,
    /// Local user that created the intent.
    pub created_by: LocalUserId,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub created_at: OffsetDateTime,
}

/// Normalized order candidate eligible only for preview.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ValidatedOrder {
    /// Validated order id.
    pub validated_order_id: ValidatedOrderId,
    /// Source preview id authorized by approvals.
    pub preview_id: OrderPreviewId,
    /// Source intent id.
    pub intent_id: OrderIntentId,
    /// Account id.
    pub account_id: AccountId,
    /// Resolved broker contract id.
    pub contract_id: ContractId,
    /// Resolved trading symbol used by later live policy checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Resolved asset class used by later live policy checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_class: Option<AssetClass>,
    /// Side.
    pub side: OrderSide,
    /// Quantity.
    pub quantity: Quantity,
    /// Order type.
    pub order_type: PreviewOrderType,
    /// Limit price.
    pub limit_price: Option<Money>,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Expiration timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub expires_at: OffsetDateTime,
    /// Safe validation warnings.
    pub warnings: Vec<String>,
}

/// Non-executable order preview result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OrderPreview {
    /// Preview id.
    pub preview_id: OrderPreviewId,
    /// Source validated order id.
    pub validated_order_id: ValidatedOrderId,
    /// Estimated cost.
    pub estimated_cost: Money,
    /// Estimated commission.
    pub estimated_commission: Option<Money>,
    /// Estimated margin impact.
    pub margin_impact: Option<Money>,
    /// Safe warnings.
    pub warnings: Vec<String>,
    /// Preview expiration timestamp.
    #[serde(with = "time::serde::rfc3339")]
    #[schemars(with = "String")]
    pub expires_at: OffsetDateTime,
    /// Audit event id.
    pub audit_event_id: AuditEventId,
}
