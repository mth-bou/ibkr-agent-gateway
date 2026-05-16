//! Risk policy and deterministic risk result models.

use crate::internal::domain::{AccountMode, AssetClass, Money, Quantity};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Deterministic risk policy for preview-only order candidates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RiskPolicy {
    /// Stable policy id.
    pub policy_id: String,
    /// Whether preview risk checks are enabled.
    pub enabled: bool,
    /// Allowed account modes.
    pub allowed_account_modes: Vec<AccountMode>,
    /// Allowed asset classes.
    pub allowed_asset_classes: Vec<AssetClass>,
    /// Maximum notional.
    pub max_notional: Option<Money>,
    /// Maximum quantity.
    pub max_quantity: Option<Quantity>,
    /// Maximum order count per future time window.
    pub max_order_count_per_window: Option<u32>,
    /// Whether fractional quantities are allowed.
    pub allow_fractional: bool,
    /// Whether fresh market data is required.
    pub requires_market_data_freshness: bool,
    /// Whether preview itself requires approval.
    pub requires_approval_for_preview: bool,
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            policy_id: "default-preview-disabled".to_string(),
            enabled: false,
            allowed_account_modes: vec![AccountMode::Paper],
            allowed_asset_classes: vec![AssetClass::Stock, AssetClass::Etf],
            max_notional: None,
            max_quantity: None,
            max_order_count_per_window: None,
            allow_fractional: false,
            requires_market_data_freshness: true,
            requires_approval_for_preview: false,
        }
    }
}

/// Non-fatal risk warning.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RiskWarning {
    /// Stable warning code.
    pub code: String,
    /// Safe warning message.
    pub message: String,
}

/// Fatal risk refusal.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RiskRefusal {
    /// Stable refusal code.
    pub code: String,
    /// Safe refusal message.
    pub message: String,
    /// Safe user action.
    pub user_action: Option<String>,
}

/// Deterministic risk decision.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskDecision {
    /// Checks passed with optional warnings.
    Allow { warnings: Vec<RiskWarning> },
    /// Checks failed closed.
    Refuse { refusals: Vec<RiskRefusal> },
}

#[cfg(test)]
mod tests {
    use super::super::run_risk_checks;
    use super::RiskPolicy;
    use crate::internal::domain::{
        AccountId, AccountMode, AssetClass, CurrencyCode, LocalUserId, Money, OrderContractInput,
        OrderIntent, OrderIntentId, OrderSide, PreviewOrderType, Quantity, TimeInForce,
    };
    use rust_decimal::Decimal;
    use time::OffsetDateTime;

    #[test]
    fn default_policy_refuses_preview() {
        let policy = RiskPolicy::default();
        let decision = run_risk_checks(&intent(), &policy);

        assert!(matches!(decision, super::RiskDecision::Refuse { .. }));
    }

    #[test]
    fn enabled_policy_allows_basic_limit_stock() {
        let policy = RiskPolicy {
            enabled: true,
            ..RiskPolicy::default()
        };
        let decision = run_risk_checks(&intent(), &policy);

        assert!(matches!(decision, super::RiskDecision::Allow { .. }));
    }

    fn intent() -> OrderIntent {
        let Some(account_id) = AccountId::new("DU1234567") else {
            unreachable!("static account id should be valid");
        };
        let Some(currency) = CurrencyCode::new("USD") else {
            unreachable!("static currency should be valid");
        };

        OrderIntent {
            intent_id: OrderIntentId::new(),
            account_id,
            account_mode: AccountMode::Paper,
            contract: OrderContractInput::Query {
                symbol: "AAPL".to_string(),
                asset_class: AssetClass::Stock,
                currency: currency.clone(),
                exchange: Some("SMART".to_string()),
            },
            side: OrderSide::Buy,
            quantity: Quantity::new(Decimal::ONE),
            order_type: PreviewOrderType::Limit,
            limit_price: Some(Money {
                amount: Decimal::new(100, 0),
                currency,
            }),
            time_in_force: TimeInForce::Day,
            rationale: None,
            created_by: LocalUserId::from_static("local-user"),
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}
