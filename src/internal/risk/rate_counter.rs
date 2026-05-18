//! Server-side live rate counters.

use super::{LiveLimitContext, LiveLimitPolicy};
use crate::internal::audit::SqliteAuditWriter;
use crate::internal::domain::{AccountId, GatewayError};

/// Applies durable server-side live submit counters to a limit context.
pub async fn apply_live_rate_counters(
    audit_writer: &SqliteAuditWriter,
    account_id: &AccountId,
    policy: &LiveLimitPolicy,
    context: &mut LiveLimitContext,
) -> Result<(), GatewayError> {
    let session_currency = context
        .session_notional
        .as_ref()
        .map(|notional| &notional.currency);
    let counts = audit_writer
        .live_rate_counts_with_session_currency(
            account_id,
            policy
                .frequency_limit
                .map(|frequency| frequency.window_seconds),
            session_currency,
        )
        .await?;
    context.submitted_in_window = counts.submitted_in_window;
    context.submitted_in_session = counts.submitted_in_session;
    if counts.session_notional.is_some() {
        context.session_notional = counts.session_notional;
    }
    Ok(())
}
