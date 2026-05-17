#![allow(dead_code)]

use ibkr_agent_gateway::testing::approval::{ApprovalId, ApprovalRecord, ApprovalStatus};
use ibkr_agent_gateway::testing::config::LiveTradingConfig;
use ibkr_agent_gateway::testing::domain::{
    AccountId, AssetClass, ContractId, CurrencyCode, ErrorCode, GatewayError, LocalUserId,
    MarketDataStatus, MarketSnapshot, Money, OrderIntentId, OrderPreviewId, OrderSide,
    PreviewOrderType, Quantity, TimeInForce, ValidatedOrder, ValidatedOrderId,
};
use ibkr_agent_gateway::testing::orders::{
    IdempotencyKey, KillSwitch, LiveSubmitRequest, PaperToLiveMigrationChecklist,
};
use ibkr_agent_gateway::testing::risk::{
    LiveFrequencyLimit, LiveLimitContext, LiveLimitPolicy, LiveSessionLimit, StaticPolicyRegistry,
};
use rust_decimal::Decimal;
use time::{Duration, OffsetDateTime};

pub const POLICY_ID: &str = "test-live-policy";

pub fn account_id() -> AccountId {
    AccountId::from_static("U1234567")
}

pub fn live_submit_request() -> Result<LiveSubmitRequest, GatewayError> {
    let account_id = account_id();
    let order = validated_order(account_id.clone())?;
    Ok(LiveSubmitRequest {
        approval: approval_for_order(account_id.clone(), &order),
        order,
        idempotency_key: IdempotencyKey::new("live-submit-key")?,
        live_config: live_config(account_id),
        live_scope_granted: true,
        live_limit_context: live_limit_context()?,
        kill_switch: KillSwitch::open(LocalUserId::from_static("operator"), "test open"),
        audit_available: true,
        migration_checklist: PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static(
            "operator",
        )),
    })
}

pub fn live_policy_registry() -> Result<StaticPolicyRegistry, GatewayError> {
    Ok(StaticPolicyRegistry::single(live_limit_policy()?))
}

pub fn live_config(account_id: AccountId) -> LiveTradingConfig {
    LiveTradingConfig {
        enabled: true,
        allowed_accounts: vec![account_id],
        risk_policy_id: Some(POLICY_ID.to_string()),
        paper_to_live_checklist_acknowledged: true,
    }
}

pub fn approval(account_id: AccountId) -> ApprovalRecord {
    ApprovalRecord {
        approval_id: ApprovalId::new(),
        preview_id: OrderPreviewId::new(),
        account_id,
        approved_by: LocalUserId::from_static("operator"),
        status: ApprovalStatus::Approved,
        approved_at: Some(OffsetDateTime::now_utc()),
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    }
}

pub fn approval_for_order(account_id: AccountId, order: &ValidatedOrder) -> ApprovalRecord {
    let mut approval = approval(account_id);
    approval.preview_id = order.preview_id.clone();
    approval
}

pub fn validated_order(account_id: AccountId) -> Result<ValidatedOrder, GatewayError> {
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
        account_id,
        contract_id: ContractId::from_static("265598"),
        side: OrderSide::Buy,
        quantity: Quantity::new(Decimal::ONE),
        order_type: PreviewOrderType::Limit,
        limit_price: Some(Money {
            amount: Decimal::new(100, 0),
            currency,
        }),
        time_in_force: TimeInForce::Day,
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
        warnings: Vec::new(),
    })
}

pub fn live_limit_policy() -> Result<LiveLimitPolicy, GatewayError> {
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Static currency is invalid",
            false,
            None,
        ));
    };

    Ok(LiveLimitPolicy {
        policy_id: POLICY_ID.to_string(),
        enabled: true,
        max_notional: Some(Money {
            amount: Decimal::new(1_000, 0),
            currency: currency.clone(),
        }),
        max_quantity: Some(Quantity::new(Decimal::new(10, 0))),
        allowed_symbols: vec!["AAPL".to_string()],
        allowed_asset_classes: vec![AssetClass::Stock],
        frequency_limit: Some(LiveFrequencyLimit {
            max_orders: 5,
            window_seconds: 300,
        }),
        session_limit: Some(LiveSessionLimit {
            max_orders_per_session: 20,
            max_session_notional: Some(Money {
                amount: Decimal::new(5_000, 0),
                currency,
            }),
        }),
        max_price_deviation_bps: Some(500),
        max_quote_age_seconds: Some(30),
    })
}

pub fn live_limit_context() -> Result<LiveLimitContext, GatewayError> {
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Static currency is invalid",
            false,
            None,
        ));
    };

    Ok(LiveLimitContext {
        symbol: "AAPL".to_string(),
        asset_class: AssetClass::Stock,
        submitted_in_window: 0,
        submitted_in_session: 0,
        session_notional: Some(Money {
            amount: Decimal::ZERO,
            currency,
        }),
        market_snapshot: Some(market_snapshot()),
    })
}

pub fn market_snapshot() -> MarketSnapshot {
    let currency = CurrencyCode::new("USD").unwrap_or_else(|| unreachable!("valid currency"));
    MarketSnapshot {
        contract_id: ContractId::from_static("265598"),
        bid: Some(Money {
            amount: Decimal::new(99, 0),
            currency: currency.clone(),
        }),
        ask: Some(Money {
            amount: Decimal::new(101, 0),
            currency: currency.clone(),
        }),
        last: Some(Money {
            amount: Decimal::new(100, 0),
            currency: currency.clone(),
        }),
        currency,
        source_timestamp: OffsetDateTime::now_utc(),
        received_at: OffsetDateTime::now_utc(),
        data_status: MarketDataStatus::Live,
        staleness_seconds: 1,
        warnings: Vec::new(),
    }
}
