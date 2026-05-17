//! Server-side live policy registry.

use super::LiveLimitPolicy;
use crate::internal::domain::{ErrorCode, GatewayError};
use async_trait::async_trait;
use std::collections::BTreeMap;

/// Server-owned source of live trading policies.
#[async_trait]
pub trait LivePolicyRegistry: Send + Sync {
    /// Loads a live policy by stable id.
    async fn load_policy(&self, policy_id: &str) -> Result<LiveLimitPolicy, GatewayError>;
}

/// Static in-process registry loaded from trusted server configuration.
#[derive(Clone, Debug, Default)]
pub struct StaticPolicyRegistry {
    policies: BTreeMap<String, LiveLimitPolicy>,
}

impl StaticPolicyRegistry {
    /// Creates a registry from server-owned policies.
    #[must_use]
    pub fn new(policies: impl IntoIterator<Item = LiveLimitPolicy>) -> Self {
        let policies = policies
            .into_iter()
            .map(|policy| (policy.policy_id.clone(), policy))
            .collect();
        Self { policies }
    }

    /// Creates a registry containing one policy.
    #[must_use]
    pub fn single(policy: LiveLimitPolicy) -> Self {
        Self::new([policy])
    }
}

#[async_trait]
impl LivePolicyRegistry for StaticPolicyRegistry {
    async fn load_policy(&self, policy_id: &str) -> Result<LiveLimitPolicy, GatewayError> {
        self.policies.get(policy_id).cloned().ok_or_else(|| {
            GatewayError::new(
                ErrorCode::LivePolicyUnknown,
                format!("Unknown live risk policy: {policy_id}"),
                false,
                Some("Configure a server-side live policy with this id".to_string()),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{LivePolicyRegistry, StaticPolicyRegistry};
    use crate::internal::domain::ErrorCode;
    use crate::internal::risk::LiveLimitPolicy;

    #[tokio::test]
    async fn static_registry_loads_policy_by_id() {
        let policy = LiveLimitPolicy {
            policy_id: "policy-a".to_string(),
            ..LiveLimitPolicy::default()
        };
        let registry = StaticPolicyRegistry::single(policy.clone());
        let loaded = registry.load_policy("policy-a").await;
        assert_eq!(loaded, Ok(policy));
    }

    #[tokio::test]
    async fn static_registry_refuses_unknown_policy_id() {
        let registry = StaticPolicyRegistry::default();
        let Err(error) = registry.load_policy("missing").await else {
            unreachable!("unknown policy id must be refused");
        };
        assert_eq!(error.code, ErrorCode::LivePolicyUnknown);
    }
}
