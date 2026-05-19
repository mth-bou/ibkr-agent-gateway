//! Order preview command.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use crate::internal::audit::SqliteAuditWriter;
use crate::internal::backend::IbkrBackend;
use crate::internal::config::OrderPreviewConfig;
use crate::internal::domain::{
    AccountMode, AssetClass, CurrencyCode, ErrorCode, GatewayError, LocalUserId, Money,
    OrderContractInput, OrderIntent, OrderIntentId, OrderSide, PreviewOrderType, Quantity,
    TimeInForce,
};
use crate::internal::orders::{build_validated_order, create_order_preview};
use crate::internal::risk::{RiskDecision, RiskPolicy, validate_order_intent};
use rust_decimal::Decimal;
use std::str::FromStr;
use time::OffsetDateTime;

/// CLI request for preview.
pub struct PreviewRequest<'a> {
    /// Account id.
    pub account: &'a str,
    /// Symbol.
    pub symbol: &'a str,
    /// Side.
    pub side: &'a str,
    /// Quantity.
    pub quantity: &'a str,
    /// Limit price.
    pub limit_price: &'a str,
    /// Currency.
    pub currency: &'a str,
    /// Explicit preview enablement.
    pub enable_preview: bool,
}

/// Runs `ibkr-agent orders preview`.
pub async fn preview(
    audit_writer: &SqliteAuditWriter,
    backend: &dyn IbkrBackend,
    request: PreviewRequest<'_>,
    json: bool,
) -> Result<(), GatewayError> {
    let config = OrderPreviewConfig {
        enabled: request.enable_preview,
        ..OrderPreviewConfig::default()
    };
    if !config.enabled {
        return Err(GatewayError::new(
            ErrorCode::OrderPreviewDisabled,
            "Order preview is disabled by default",
            false,
            Some("Enable preview explicitly for this local run".to_string()),
        ));
    }

    let account_id = parse_account_id(request.account)?;
    let quantity = parse_decimal(request.quantity, "quantity")?;
    let limit_price = parse_decimal(request.limit_price, "limit price")?;
    let Some(currency) = CurrencyCode::new(request.currency) else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Currency must be a three-letter code",
            false,
            Some("Use a valid ISO currency code".to_string()),
        ));
    };

    let side = parse_side(request.side)?;
    let intent = OrderIntent {
        intent_id: OrderIntentId::new(),
        account_id: account_id.clone(),
        account_mode: AccountMode::Paper,
        contract: OrderContractInput::Query {
            symbol: request.symbol.to_string(),
            asset_class: AssetClass::Stock,
            currency: currency.clone(),
            exchange: Some("SMART".to_string()),
        },
        side,
        quantity: Quantity::new(quantity),
        order_type: PreviewOrderType::Limit,
        limit_price: Some(Money {
            amount: limit_price,
            currency,
        }),
        stop_price: None,
        trailing_amount: None,
        trailing_percent: None,
        time_in_force: TimeInForce::Day,
        rationale: None,
        created_by: LocalUserId::from_static("local-user"),
        created_at: OffsetDateTime::now_utc(),
    };

    let policy = RiskPolicy {
        enabled: true,
        ..RiskPolicy::default()
    };
    let decision = validate_order_intent(&intent, &policy)?;
    let RiskDecision::Allow { warnings } = decision else {
        return Err(GatewayError::new(
            ErrorCode::OrderPolicyRefused,
            "Order preview was refused by deterministic risk policy",
            false,
            Some("Inspect risk refusal details".to_string()),
        ));
    };

    let contract = backend.resolve_contract(request.symbol).await?;
    let validated = build_validated_order(
        &intent,
        &contract,
        warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect(),
        config.preview_expiration_seconds,
    )?;
    let preview = create_order_preview(
        &validated,
        crate::internal::domain::AuditEventId::new(),
        None,
        None,
    )?;
    audit_writer
        .append_order_preview(&preview, &validated)
        .await?;
    print_output(json, "order preview created", &preview)
}

fn parse_decimal(value: &str, label: &str) -> Result<Decimal, GatewayError> {
    Decimal::from_str(value).map_err(|_| {
        GatewayError::new(
            ErrorCode::OrderValidationFailed,
            format!("Invalid decimal {label}"),
            false,
            Some(format!("Provide a valid decimal {label}")),
        )
    })
}

fn parse_side(value: &str) -> Result<OrderSide, GatewayError> {
    match value {
        "buy" => Ok(OrderSide::Buy),
        "sell" => Ok(OrderSide::Sell),
        _ => Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Side must be buy or sell",
            false,
            Some("Use --side buy or --side sell".to_string()),
        )),
    }
}
