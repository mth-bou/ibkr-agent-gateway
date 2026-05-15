//! Live order commands.

use crate::{commands::account::parse_account_id, output::print_output};
use ibkr_approval::{ApprovalId, ApprovalRecord, ApprovalStatus};
use ibkr_config::LiveTradingConfig;
use ibkr_domain::{
    AssetClass, BrokerOrderId, ContractId, CurrencyCode, ErrorCode, GatewayError, LocalUserId,
    Money, OrderIntentId, OrderSide, PreviewOrderType, Quantity, TimeInForce, ValidatedOrder,
    ValidatedOrderId,
};
use ibkr_orders::{
    IdempotencyKey, IdempotencyStore, KillSwitch, LiveCancelRequest, LiveSubmitRequest,
    PaperToLiveMigrationChecklist, cancel_live_order, submit_live_order,
};
use ibkr_risk::{LiveFrequencyLimit, LiveLimitContext, LiveLimitPolicy, LiveSessionLimit};
use rust_decimal::Decimal;
use time::{Duration, OffsetDateTime};

/// Runs a live submit command.
pub fn submit(
    account: &str,
    idempotency_key: &str,
    gates: LiveCommandGates,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let request = LiveSubmitRequest {
        order: dummy_validated_order(account_id.clone())?,
        approval: dummy_approval(account_id.clone()),
        idempotency_key: IdempotencyKey::new(idempotency_key)?,
        live_config: live_config(account_id, gates.enable_live, gates.acknowledge_migration),
        live_scope_granted: gates.live_scope,
        live_limit_policy: live_limit_policy()?,
        live_limit_context: live_limit_context()?,
        kill_switch: kill_switch(gates.open_kill_switch),
        audit_available: true,
        migration_checklist: migration_checklist(gates.acknowledge_migration),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let result = submit_live_order(request, &mut idempotency_store)?;
    print_output(json, "live order submitted", &result.lifecycle)
}

/// Runs a live cancel command.
pub fn cancel(
    account: &str,
    broker_order_id: &str,
    idempotency_key: &str,
    gates: LiveCommandGates,
    json: bool,
) -> Result<(), GatewayError> {
    let account_id = parse_account_id(account)?;
    let Some(broker_order_id) = BrokerOrderId::new(broker_order_id) else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Broker order id is required",
            false,
            Some("Provide a broker order id".to_string()),
        ));
    };
    let request = LiveCancelRequest {
        account_id: account_id.clone(),
        broker_order_id,
        idempotency_key: IdempotencyKey::new(idempotency_key)?,
        live_config: live_config(account_id, gates.enable_live, gates.acknowledge_migration),
        live_scope_granted: gates.live_scope,
        kill_switch: kill_switch(gates.open_kill_switch),
        audit_available: true,
        migration_checklist: migration_checklist(gates.acknowledge_migration),
    };
    let mut idempotency_store = IdempotencyStore::default();
    let result = cancel_live_order(request, &mut idempotency_store)?;
    print_output(json, "live order cancelled", &result.lifecycle)
}

/// Live CLI gate flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LiveCommandGates {
    /// Explicit live enablement.
    pub enable_live: bool,
    /// Simulated live scope.
    pub live_scope: bool,
    /// Simulated kill switch state.
    pub open_kill_switch: bool,
    /// Paper-to-live checklist acknowledgement.
    pub acknowledge_migration: bool,
}

fn live_config(
    account_id: ibkr_domain::AccountId,
    enabled: bool,
    migration_acknowledged: bool,
) -> LiveTradingConfig {
    LiveTradingConfig {
        enabled,
        allowed_accounts: vec![account_id],
        risk_policy_id: Some("cli-live-policy".to_string()),
        paper_to_live_checklist_acknowledged: migration_acknowledged,
    }
}

fn kill_switch(open: bool) -> KillSwitch {
    if open {
        KillSwitch::open(LocalUserId::from_static("local-user"), "cli live smoke")
    } else {
        KillSwitch::closed(LocalUserId::from_static("local-user"), "closed by default")
    }
}

fn migration_checklist(acknowledged: bool) -> PaperToLiveMigrationChecklist {
    if acknowledged {
        PaperToLiveMigrationChecklist::acknowledged(LocalUserId::from_static("local-user"))
    } else {
        PaperToLiveMigrationChecklist {
            paper_trading_validated: false,
            approvals_reviewed: false,
            limits_reviewed: false,
            kill_switch_tested: false,
            incident_runbook_reviewed: false,
            acknowledged_by: None,
            acknowledged_at: None,
        }
    }
}

fn dummy_approval(account_id: ibkr_domain::AccountId) -> ApprovalRecord {
    ApprovalRecord {
        approval_id: ApprovalId::new(),
        preview_id: ibkr_domain::OrderPreviewId::new(),
        account_id,
        approved_by: LocalUserId::from_static("local-user"),
        status: ApprovalStatus::Approved,
        approved_at: Some(OffsetDateTime::now_utc()),
        expires_at: OffsetDateTime::now_utc() + Duration::minutes(5),
    }
}

fn dummy_validated_order(
    account_id: ibkr_domain::AccountId,
) -> Result<ValidatedOrder, GatewayError> {
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

fn live_limit_policy() -> Result<LiveLimitPolicy, GatewayError> {
    let Some(currency) = CurrencyCode::new("USD") else {
        return Err(GatewayError::new(
            ErrorCode::OrderValidationFailed,
            "Static currency is invalid",
            false,
            None,
        ));
    };

    Ok(LiveLimitPolicy {
        policy_id: "cli-live-policy".to_string(),
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
    })
}

fn live_limit_context() -> Result<LiveLimitContext, GatewayError> {
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
    })
}
