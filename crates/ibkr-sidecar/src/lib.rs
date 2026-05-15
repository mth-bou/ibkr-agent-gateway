//! Local sidecar relay primitives for retail Client Portal Gateway sessions.

pub mod audit;
pub mod client_portal_forwarder;
pub mod heartbeat;
pub mod identity;
pub mod pairing;
pub mod relay;
pub mod session_state;

pub use audit::build_sidecar_relay_audit_event;
pub use client_portal_forwarder::{ForwardedBrokerResponse, build_forwarded_broker_request};
pub use heartbeat::{SidecarHeartbeat, apply_heartbeat};
pub use identity::{SidecarCapability, SidecarId, SidecarIdentity};
pub use pairing::{PairingId, PairingRecord, PairingStatus, create_pairing};
pub use relay::{ForwardedBrokerRequest, RelaySession, RelaySessionId, create_relay_session};
pub use session_state::{RelayAvailability, require_available_session};
