use ibkr_agent_gateway::testing::backend::{
    FakeBackend, FakeFixtureStore, IbkrBackend, apply_market_data_policy,
};
use ibkr_agent_gateway::testing::cpapi::{CpapiContractCandidate, map_contract_candidate};
use ibkr_agent_gateway::testing::domain::{
    ContractId, ErrorCode, HistoricalBarsRequest, MarketDataPolicy, MarketDataStatus, StalePolicy,
};

#[tokio::test]
async fn fake_backend_contracts_market_and_bars_work() -> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let candidates = backend.search_contracts("AAPL").await?;
    assert_eq!(candidates.len(), 1);

    let resolved = backend.resolve_contract("AAPL").await?;
    assert_eq!(resolved.symbol, "AAPL");

    let Some(contract_id) = ContractId::new("265598") else {
        return Err("contract id rejected".into());
    };
    let snapshot = backend.market_snapshot(&contract_id).await?;
    assert_eq!(snapshot.data_status, MarketDataStatus::Live);

    let bars = backend
        .historical_bars(&HistoricalBarsRequest {
            contract_id,
            duration: "1 D".to_string(),
            bar_size: "5 mins".to_string(),
            outside_regular_trading_hours: false,
        })
        .await?;
    assert_eq!(bars.len(), 1);
    Ok(())
}

#[tokio::test]
async fn fake_backend_refuses_ambiguous_contract_resolution()
-> Result<(), Box<dyn std::error::Error>> {
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));
    let error = backend.resolve_contract("AMBIG").await;
    match error {
        Err(error) => {
            assert_eq!(error.code, ErrorCode::InputAmbiguousContract);
            Ok(())
        }
        Ok(_) => Err("ambiguous contract should refuse".into()),
    }
}

#[test]
fn unsupported_asset_class_mapping_refuses() -> Result<(), Box<dyn std::error::Error>> {
    let store = FakeFixtureStore::new("tests/fixtures/cpapi");
    let candidates: Vec<CpapiContractCandidate> =
        store.load_json("contracts_unsupported_asset_class.json")?;
    let error = map_contract_candidate(candidates[0].clone());
    let Err(error) = error else {
        return Err("unsupported asset class should refuse".into());
    };
    assert_eq!(error.code, ErrorCode::InputUnsupportedAssetClass);
    Ok(())
}

#[test]
fn stale_policy_can_refuse_stale_data() {
    let policy = MarketDataPolicy {
        stale_policy: StalePolicy::Refuse,
        ..MarketDataPolicy::default()
    };
    let error = apply_market_data_policy(&policy, MarketDataStatus::Stale);
    assert!(error.is_err());
}
