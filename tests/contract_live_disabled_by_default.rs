use ibkr_auth::{HEALTH_READ, ScopeSet};
use ibkr_config::{
    AccountIdMode, AuditRetentionConfig, AuditStorageConfig, GatewayConfiguration,
    LiveTradingConfig, OrderPreviewConfig, PaperTradingConfig, RemoteMcpConfig, SafetyConfig,
    ServerMode, SidecarConfig,
};
use ibkr_domain::{BrokerBackendKind, ErrorCode, MarketDataPolicy};

#[test]
fn live_config_and_safety_are_disabled_by_default() {
    assert!(!LiveTradingConfig::default().enabled);
    assert!(!SafetyConfig::default().live_trading_enabled);
}

#[test]
fn safety_live_flag_without_live_config_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let config = gateway_config(SafetyConfig {
        live_trading_enabled: true,
        ..SafetyConfig::default()
    })?;

    let error = config
        .validate()
        .expect_err("live safety flag alone must fail closed");

    assert_eq!(error.code, ErrorCode::ConfigLiveTradingForbidden);
    Ok(())
}

fn gateway_config(
    safety: SafetyConfig,
) -> Result<GatewayConfiguration, Box<dyn std::error::Error>> {
    Ok(GatewayConfiguration {
        server_mode: ServerMode::Local,
        bind_address: "127.0.0.1:8080".to_string(),
        broker_backend: BrokerBackendKind::Fake,
        client_portal_base_url: None,
        verify_tls: true,
        keepalive_interval_seconds: 60,
        audit_storage: AuditStorageConfig::Sqlite {
            storage: "sqlite::memory:".to_string(),
        },
        audit_account_id_mode: AccountIdMode::Hmac,
        audit_retention: AuditRetentionConfig::default(),
        enabled_read_scopes: ScopeSet::read_only([HEALTH_READ])?,
        market_data_policy: MarketDataPolicy::default(),
        order_preview: OrderPreviewConfig::default(),
        paper_trading: PaperTradingConfig::default(),
        live_trading: LiveTradingConfig::default(),
        remote_mcp: RemoteMcpConfig::default(),
        sidecar: SidecarConfig::default(),
        safety,
    })
}
