//! Typed identifiers and correlation values.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stable local user identifier used for audit correlation.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct LocalUserId(String);

/// Broker account identifier as returned by IBKR.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct AccountId(String);

/// HMAC-SHA256 account correlation value used in audit events.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct AccountIdHash(String);

/// Broker contract identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ContractId(String);

/// Broker order identifier for read-only order records.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct BrokerOrderId(String);

/// Request correlation identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct RequestId(Uuid);

/// Local session correlation identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct SessionId(Uuid);

/// Audit event identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct AuditEventId(Uuid);

macro_rules! impl_string_id {
    ($type_name:ident) => {
        impl $type_name {
            /// Creates a non-empty string identifier.
            ///
            /// Returns `None` when the value is empty after trimming.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Option<Self> {
                let value = value.into();
                if value.trim().is_empty() {
                    None
                } else {
                    Some(Self(value))
                }
            }

            /// Returns the raw identifier value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Creates an identifier from a trusted static non-empty value.
            #[must_use]
            pub fn from_static(value: &'static str) -> Self {
                Self(value.to_string())
            }
        }
    };
}

impl_string_id!(LocalUserId);
impl_string_id!(AccountId);
impl_string_id!(AccountIdHash);
impl_string_id!(ContractId);
impl_string_id!(BrokerOrderId);

impl AccountIdHash {
    /// Creates a hash identifier from computed hash material.
    #[must_use]
    pub fn from_hash(value: String) -> Self {
        Self(value)
    }
}

macro_rules! impl_uuid_id {
    ($type_name:ident) => {
        impl $type_name {
            /// Creates a fresh UUID-backed identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Creates an identifier from an existing UUID.
            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            /// Returns the inner UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

impl_uuid_id!(RequestId);
impl_uuid_id!(SessionId);
impl_uuid_id!(AuditEventId);
