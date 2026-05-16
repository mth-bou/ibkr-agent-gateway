//! Hidden exports used by integration tests during the packaging migration.

pub mod approval {
    pub use crate::internal::approval::*;
}

pub mod audit {
    pub use crate::internal::audit::*;
}

pub mod auth {
    pub use crate::internal::auth::*;
}

pub mod backend {
    pub use crate::internal::backend::*;
}

pub mod config {
    pub use crate::internal::config::*;
}

pub mod cpapi {
    pub use crate::internal::cpapi::*;
}

pub mod domain {
    pub use crate::internal::domain::*;
}

pub mod mcp {
    pub use crate::internal::mcp::*;
}

pub mod oauth {
    pub use crate::internal::oauth::*;
}

pub mod observability {
    pub use crate::internal::observability::*;
}

pub mod orders {
    pub use crate::internal::orders::*;
}

pub mod provider_compat {
    pub use crate::internal::provider_compat::*;
}

pub mod risk {
    pub use crate::internal::risk::*;
}

pub mod sidecar {
    pub use crate::internal::sidecar::*;
}
