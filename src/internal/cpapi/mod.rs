//! Interactive Brokers Client Portal Gateway adapter crate.

pub mod client;
pub mod mapper;
pub mod models;
pub mod orders_write;
pub mod preview;

pub use client::ClientPortalClient;
pub use mapper::{map_account, map_contract_candidate, map_session_response, map_tickle_response};
pub use models::{
    CpapiAccount, CpapiAccountsResponse, CpapiContractCandidate, CpapiContractsResponse,
    CpapiExecutionsResponse, CpapiHistoricalBarsResponse, CpapiJsonResponse,
    CpapiMarketSnapshotResponse, CpapiOrdersResponse, CpapiSessionResponse, CpapiTickleResponse,
};
