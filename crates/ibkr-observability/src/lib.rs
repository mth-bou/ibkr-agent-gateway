//! Compatibility crate for observability helpers while root migration is in progress.

#[path = "../../../src/internal/observability/logs.rs"]
pub mod logs;
#[path = "../../../src/internal/observability/metrics.rs"]
pub mod metrics;

pub use logs::{StructuredLogEvent, sanitize_log_fields};
pub use metrics::{MetricEvent, validate_metric_event};
