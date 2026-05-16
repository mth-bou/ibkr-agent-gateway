//! Public SDK facade for the `ibkr-agent-gateway` package.

#[doc(hidden)]
pub mod cli;
mod internal;
mod public;
#[doc(hidden)]
pub mod testing;

pub mod audit {
    //! Audit models, redaction helpers, and persistence entrypoints.

    pub use crate::public::audit::*;
}

pub mod config {
    //! Runtime configuration types and validation helpers.

    pub use crate::public::config::*;
}

pub mod mcp {
    //! MCP tool registry and transport helpers.

    pub use crate::public::mcp::*;
}

pub mod orders {
    //! Order preview, paper/live gating, and risk policy helpers.

    pub use crate::public::orders::*;
}

pub mod prelude {
    //! Common imports for SDK consumers.

    pub use crate::public::prelude::*;
}

pub use public::gateway::{Gateway, GatewayConfig};

/// Marker used by integration tests to prove the harness is available.
pub const HARNESS_NAME: &str = "ibkr-agent-gateway";
