//! Audit review commands.

use crate::cli::output::print_output;
use crate::internal::audit::{AuditHmacKey, AuditTailRequest, SqliteAuditWriter};
use crate::internal::domain::{ErrorCode, GatewayError};
use std::sync::Arc;

/// Reads recent audit events from SQLite.
pub async fn tail(
    runtime_writer: &SqliteAuditWriter,
    database_url: &str,
    limit: u32,
    json: bool,
) -> Result<(), GatewayError> {
    let output = if database_url.trim().is_empty() {
        runtime_writer
            .tail_verified(AuditTailRequest::new(limit))
            .await?
    } else {
        let writer = SqliteAuditWriter::connect(database_url, cli_audit_key()?).await?;
        writer.tail(AuditTailRequest::new(limit)).await?
    };
    print_output(json, "audit tail loaded", &output)
}

/// Exports recent audit events as JSONL.
pub async fn export(
    runtime_writer: &SqliteAuditWriter,
    database_url: &str,
    limit: u32,
    json: bool,
) -> Result<(), GatewayError> {
    let output = if database_url.trim().is_empty() {
        runtime_writer
            .export_jsonl(AuditTailRequest::new(limit))
            .await?
    } else {
        let writer = SqliteAuditWriter::connect(database_url, cli_audit_key()?).await?;
        writer.export_jsonl(AuditTailRequest::new(limit)).await?
    };
    if json {
        print_output(true, "audit export loaded", &output)
    } else {
        print!("{}", output.payload_jsonl);
        Ok(())
    }
}

/// Verifies the full chained HMAC audit log.
pub async fn verify(
    runtime_writer: &SqliteAuditWriter,
    database_url: &str,
    hmac_secret_env: &str,
    json: bool,
) -> Result<(), GatewayError> {
    let output = if database_url.trim().is_empty() {
        runtime_writer.verify_chain().await?
    } else {
        if hmac_secret_env.trim().is_empty() {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                "External audit verification requires an audit HMAC key",
                false,
                Some("Pass --hmac-secret-env with the original audit HMAC secret".to_string()),
            ));
        }
        let writer =
            SqliteAuditWriter::connect(database_url, audit_key_from_env(hmac_secret_env)?).await?;
        writer.verify_chain().await?
    };
    print_output(json, "audit chain verified", &output)?;
    if output.chain_valid {
        Ok(())
    } else {
        Err(GatewayError::new(
            ErrorCode::AuditChainInvalid,
            "Audit chain verification failed",
            false,
            Some("Investigate local audit storage for tampering".to_string()),
        ))
    }
}

/// CLI audit commands only read existing rows; verification requires the
/// original write-time key, which is operator-supplied configuration not yet
/// wired into the CLI. Use an ephemeral key so the writer can still load and
/// surface stored events even though it cannot itself verify chain integrity.
fn cli_audit_key() -> Result<Arc<AuditHmacKey>, GatewayError> {
    Ok(Arc::new(AuditHmacKey::ephemeral()?))
}

fn audit_key_from_env(env_name: &str) -> Result<Arc<AuditHmacKey>, GatewayError> {
    let value = std::env::var(env_name).map_err(|_| {
        GatewayError::new(
            ErrorCode::ConfigInvalid,
            format!("Required audit HMAC env var is not set: {env_name}"),
            false,
            Some(format!(
                "Set {env_name} before verifying this audit database"
            )),
        )
    })?;
    Ok(Arc::new(AuditHmacKey::new(value.into_bytes())?))
}
