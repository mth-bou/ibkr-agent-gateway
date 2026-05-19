//! Bracket preview construction.

use super::create_order_preview;
use crate::internal::domain::{
    AuditEventId, BracketOrderPreview, GatewayError, ValidatedOrderGroup,
};

/// Creates a non-executable three-leg bracket preview.
pub fn create_bracket_order_preview(
    group: &ValidatedOrderGroup,
) -> Result<BracketOrderPreview, GatewayError> {
    Ok(BracketOrderPreview {
        group_id: group.group_id.clone(),
        parent: create_order_preview(&group.parent, AuditEventId::new(), None, None)?,
        take_profit: create_order_preview(&group.take_profit, AuditEventId::new(), None, None)?,
        stop_loss: create_order_preview(&group.stop_loss, AuditEventId::new(), None, None)?,
    })
}
