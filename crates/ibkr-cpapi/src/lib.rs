//! Interactive Brokers Client Portal Gateway adapter crate.

pub mod client;
pub mod mapper;
pub mod models;

pub use client::ClientPortalClient;
pub use mapper::{map_account, map_session_response, map_tickle_response};
pub use models::{CpapiAccount, CpapiAccountsResponse, CpapiSessionResponse, CpapiTickleResponse};
