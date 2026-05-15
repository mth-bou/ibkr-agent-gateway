//! Local sidecar relay primitives for retail Client Portal Gateway sessions.

pub mod identity;
pub mod pairing;

pub use identity::{SidecarCapability, SidecarId, SidecarIdentity};
pub use pairing::{PairingId, PairingRecord, PairingStatus, create_pairing};
