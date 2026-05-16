//! CLI output helpers.

use crate::internal::domain::GatewayError;
use serde::Serialize;

/// Prints structured or human-readable output.
pub fn print_output<T: Serialize>(json: bool, human: &str, value: &T) -> Result<(), GatewayError> {
    if json {
        let rendered = serde_json::to_string_pretty(value).map_err(|_| {
            GatewayError::new(
                crate::internal::domain::ErrorCode::OutputUnsafe,
                "Failed to serialize command output",
                false,
                Some("Retry with human-readable output".to_string()),
            )
        })?;
        println!("{rendered}");
    } else {
        println!("{human}");
    }
    Ok(())
}

/// Prints an error.
pub fn print_error(error: &GatewayError, json: bool) {
    if json {
        let rendered = serde_json::to_string_pretty(error)
            .unwrap_or_else(|_| "{\"code\":\"OUTPUT_UNSAFE\"}".to_string());
        eprintln!("{rendered}");
    } else {
        eprintln!("{:?}: {}", error.code, error.message);
        if let Some(user_action) = &error.user_action {
            eprintln!("Action: {user_action}");
        }
    }
}
