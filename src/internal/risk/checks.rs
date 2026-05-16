//! Deterministic risk checks for preview-only order intents.

use super::policy::{RiskDecision, RiskPolicy, RiskRefusal, RiskWarning};
use crate::internal::domain::{AssetClass, OrderContractInput, OrderIntent, PreviewOrderType};
use rust_decimal::Decimal;

/// Runs deterministic risk checks for an order intent.
#[must_use]
pub fn run_risk_checks(intent: &OrderIntent, policy: &RiskPolicy) -> RiskDecision {
    let mut warnings = Vec::new();
    let mut refusals = Vec::new();

    if !policy.enabled {
        refusals.push(refusal(
            "ORDER_PREVIEW_DISABLED",
            "Order preview is disabled by local policy",
            "Enable order preview explicitly in local config",
        ));
    }

    if !policy.allowed_account_modes.contains(&intent.account_mode) {
        refusals.push(refusal(
            "ORDER_ACCOUNT_MODE_REFUSED",
            "Account mode is not allowed by preview policy",
            "Use an allowed account mode",
        ));
    }

    if let Some(asset_class) = intent_asset_class(intent)
        && !policy.allowed_asset_classes.contains(&asset_class)
    {
        refusals.push(refusal(
            "ORDER_ASSET_CLASS_REFUSED",
            "Asset class is not allowed by preview policy",
            "Use a supported stock or ETF contract",
        ));
    }

    if intent.quantity.value <= Decimal::ZERO {
        refusals.push(refusal(
            "ORDER_QUANTITY_INVALID",
            "Quantity must be positive",
            "Use a positive decimal quantity",
        ));
    }

    if !policy.allow_fractional && !is_whole_quantity(intent.quantity.value) {
        refusals.push(refusal(
            "ORDER_FRACTIONAL_REFUSED",
            "Fractional quantity is not allowed by preview policy",
            "Use a whole-share quantity or enable fractional preview policy",
        ));
    }

    if let Some(max_quantity) = &policy.max_quantity
        && intent.quantity.value > max_quantity.value
    {
        refusals.push(refusal(
            "ORDER_QUANTITY_LIMIT_REFUSED",
            "Quantity exceeds preview policy maximum",
            "Reduce the requested quantity",
        ));
    }

    match intent.order_type {
        PreviewOrderType::Limit => {
            if intent.limit_price.is_none() {
                refusals.push(refusal(
                    "ORDER_LIMIT_PRICE_REQUIRED",
                    "Limit price is required for limit order preview",
                    "Provide a limit price",
                ));
            }
        }
        PreviewOrderType::Market => refusals.push(refusal(
            "ORDER_MARKET_TYPE_REFUSED",
            "Market order preview is refused by default policy",
            "Use a limit order candidate",
        )),
    }

    if let (Some(limit_price), Some(max_notional)) = (&intent.limit_price, &policy.max_notional) {
        let notional = limit_price.amount * intent.quantity.value;
        if notional > max_notional.amount {
            refusals.push(refusal(
                "ORDER_NOTIONAL_LIMIT_REFUSED",
                "Estimated notional exceeds preview policy maximum",
                "Reduce quantity or price",
            ));
        }
        if limit_price.currency != max_notional.currency {
            warnings.push(RiskWarning {
                code: "ORDER_NOTIONAL_CURRENCY_MISMATCH".to_string(),
                message: "Notional limit currency differs from limit price currency".to_string(),
            });
        }
    }

    if refusals.is_empty() {
        RiskDecision::Allow { warnings }
    } else {
        RiskDecision::Refuse { refusals }
    }
}

fn intent_asset_class(intent: &OrderIntent) -> Option<AssetClass> {
    match intent.contract {
        OrderContractInput::ContractId { .. } => None,
        OrderContractInput::Query { asset_class, .. } => Some(asset_class),
    }
}

fn is_whole_quantity(value: Decimal) -> bool {
    value.fract().is_zero()
}

fn refusal(code: &str, message: &str, user_action: &str) -> RiskRefusal {
    RiskRefusal {
        code: code.to_string(),
        message: message.to_string(),
        user_action: Some(user_action.to_string()),
    }
}
