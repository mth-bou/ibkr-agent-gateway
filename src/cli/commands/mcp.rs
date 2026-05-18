//! MCP serve command.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use crate::internal::audit::{AuditResultStatus, AuditTailRequest, SqliteAuditWriter};
use crate::internal::auth::ScopeSet;
use crate::internal::backend::IbkrBackend;
use crate::internal::config::RemoteMcpConfig;
use crate::internal::domain::{ContractId, ErrorCode, GatewayError, HistoricalBarsRequest};
use serde::Serialize;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::time::{Duration, MissedTickBehavior};

/// MCP serve output.
#[derive(Debug, Serialize)]
pub struct McpServeOutput {
    /// Transport name.
    pub transport: String,
    /// Status message.
    pub status: String,
}

/// MCP serve command options.
pub struct McpServeOptions<'a> {
    /// Transport.
    pub transport: &'a str,
    /// Print a transport description and exit.
    pub describe: bool,
    /// Whether remote HTTP MCP is explicitly enabled for this invocation.
    pub enable_remote_mcp: bool,
    /// HTTP bind address.
    pub bind: &'a str,
    /// Remote MCP config loaded from the runtime config file.
    pub remote_mcp_config: &'a RemoteMcpConfig,
    /// Emit JSON output.
    pub json: bool,
    /// Live lifecycle reconciliation interval in seconds.
    pub live_reconciler_interval_seconds: u64,
}

/// Runs `ibkr-agent mcp serve`.
pub async fn serve(
    backend: &dyn IbkrBackend,
    audit_writer: &SqliteAuditWriter,
    scopes: &ScopeSet,
    options: McpServeOptions<'_>,
) -> Result<(), GatewayError> {
    let status = match options.transport {
        "stdio" => {
            if options.describe {
                crate::internal::mcp::serve_stdio_description()
            } else {
                return serve_stdio(
                    backend,
                    audit_writer,
                    scopes,
                    options.live_reconciler_interval_seconds,
                )
                .await;
            }
        }
        "http" => {
            if !options.enable_remote_mcp {
                return Err(GatewayError::new(
                    ErrorCode::ConfigRemoteMcpForbidden,
                    "HTTP MCP requires explicit remote enablement",
                    false,
                    Some("Pass --enable-remote-mcp with complete OAuth configuration".to_string()),
                ));
            }
            if !options.remote_mcp_config.enabled {
                return Err(GatewayError::new(
                    ErrorCode::ConfigRemoteMcpForbidden,
                    "HTTP MCP requires remote_mcp.enabled in the runtime config",
                    false,
                    Some("Configure remote_mcp before passing --enable-remote-mcp".to_string()),
                ));
            }
            let mut config = options.remote_mcp_config.clone();
            config.bind_address = options.bind.to_string();
            crate::internal::mcp::serve_http_description(&config)?
        }
        _ => {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                "Unsupported MCP transport",
                false,
                Some("Use --transport stdio or --transport http".to_string()),
            ));
        }
    };
    let output = McpServeOutput {
        transport: options.transport.to_string(),
        status,
    };
    print_output(options.json, &output.status, &output)
}

async fn serve_stdio(
    backend: &dyn IbkrBackend,
    audit_writer: &SqliteAuditWriter,
    scopes: &ScopeSet,
    live_reconciler_interval_seconds: u64,
) -> Result<(), GatewayError> {
    let stdin = tokio::io::stdin();
    let mut lines = tokio::io::BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();
    let mut reconciliation_interval =
        tokio::time::interval(Duration::from_secs(live_reconciler_interval_seconds.max(1)));
    reconciliation_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    reconciliation_interval.reset();

    loop {
        tokio::select! {
            line = lines.next_line() => {
                let Some(line) = line.map_err(io_error)? else {
                    return Ok(());
                };
                if line.trim().is_empty() {
                    continue;
                }
                let response = match serde_json::from_str::<Value>(&line) {
                    Ok(request) => handle_stdio_request(backend, audit_writer, scopes, &request).await,
                    Err(_) => Some(jsonrpc_error(Value::Null, -32700, "Parse error")),
                };
                let Some(response) = response else {
                    continue;
                };
                let rendered = serde_json::to_string(&response).map_err(|_| {
                    GatewayError::new(
                        ErrorCode::OutputUnsafe,
                        "Failed to serialize MCP response",
                        false,
                        Some("Retry the MCP request".to_string()),
                    )
                })?;
                stdout
                    .write_all(rendered.as_bytes())
                    .await
                    .map_err(io_error)?;
                stdout.write_all(b"\n").await.map_err(io_error)?;
                stdout.flush().await.map_err(io_error)?;
            }
            _ = reconciliation_interval.tick() => {
                if let Err(error) = crate::internal::orders::reconcile_live_orders_once(audit_writer, backend).await {
                    tracing::warn!(
                        target: "orders.reconciler",
                        error_code = ?error.code,
                        "live order reconciliation tick failed"
                    );
                }
            }
        }
    }
}

async fn handle_stdio_request(
    backend: &dyn IbkrBackend,
    audit_writer: &SqliteAuditWriter,
    scopes: &ScopeSet,
    request: &Value,
) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return Some(jsonrpc_error(id, -32600, "Invalid Request"));
    };

    match method {
        "initialize" => Some(jsonrpc_result(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "ibkr-agent-gateway",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {}
                }
            }),
        )),
        "notifications/initialized" => None,
        "tools/list" => Some(jsonrpc_result(
            id,
            json!({
                "tools": crate::internal::mcp::broker_tool_schemas()
                    .into_iter()
                    .filter(|tool| scopes.contains(&tool.scope))
                    .map(|tool| {
                        json!({
                            "name": tool.name,
                            "inputSchema": tool.input_schema,
                            "outputSchema": tool.output_schema,
                            "annotations": { "scope": tool.scope }
                        })
                    })
                    .collect::<Vec<_>>()
            }),
        )),
        "tools/call" => Some(
            match call_tool(backend, audit_writer, scopes, request).await {
                Ok(value) => jsonrpc_result(
                    id,
                    json!({
                        "content": [{
                            "type": "text",
                            "text": value.to_string()
                        }]
                    }),
                ),
                Err(error) => {
                    jsonrpc_error(id, -32000, &format!("{:?}: {}", error.code, error.message))
                }
            },
        ),
        _ => Some(jsonrpc_error(id, -32601, "Method not found")),
    }
}

async fn call_tool(
    backend: &dyn IbkrBackend,
    audit_writer: &SqliteAuditWriter,
    scopes: &ScopeSet,
    request: &Value,
) -> Result<Value, GatewayError> {
    let params = request.get("params").unwrap_or(&Value::Null);
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_mcp_request("Tool name is required"))?;
    let args = params.get("arguments").unwrap_or(&Value::Null);

    let tool = crate::internal::mcp::registry::find_broker_tool_schema(name)
        .ok_or_else(|| unavailable_mcp_tool(name))?;
    if let Err(error) = crate::internal::mcp::enforce_scope(scopes, &tool.scope) {
        record_mcp_tool_audit(audit_writer, name, &tool.scope, &Err(error.clone())).await?;
        return Err(error);
    }

    let result = execute_tool(backend, audit_writer, name, args).await;
    record_mcp_tool_audit(audit_writer, name, &tool.scope, &result).await?;
    result
}

async fn execute_tool(
    backend: &dyn IbkrBackend,
    audit_writer: &SqliteAuditWriter,
    name: &str,
    args: &Value,
) -> Result<Value, GatewayError> {
    match name {
        "ibkr_health" => Ok(json!({ "gateway": "ok", "read_only": true })),
        "ibkr_backend_status" | "ibkr_session_requirements" => {
            Ok(serde_json::to_value(backend.session_status().await?).map_err(output_error)?)
        }
        "ibkr_accounts_list" => {
            Ok(serde_json::to_value(backend.list_accounts().await?).map_err(output_error)?)
        }
        "ibkr_account_summary" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(backend.account_summary(&account).await?)
        }
        "ibkr_positions_list" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(serde_json::to_value(backend.positions(&account).await?).map_err(output_error)?)
        }
        "ibkr_portfolio_snapshot" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(backend.portfolio_snapshot(&account).await?)
        }
        "ibkr_contracts_search" => Ok(serde_json::to_value(
            backend.search_contracts(arg_string(args, "query")?).await?,
        )
        .map_err(output_error)?),
        "ibkr_contract_resolve" => Ok(serde_json::to_value(
            backend
                .resolve_contract(arg_string(args, "symbol")?)
                .await?,
        )
        .map_err(output_error)?),
        "ibkr_market_snapshot" => {
            let contract_id = parse_contract_id(arg_string(args, "contract_id")?)?;
            Ok(
                serde_json::to_value(backend.market_snapshot(&contract_id).await?)
                    .map_err(output_error)?,
            )
        }
        "ibkr_historical_bars" => {
            let request = HistoricalBarsRequest {
                contract_id: parse_contract_id(arg_string(args, "contract_id")?)?,
                duration: arg_string(args, "duration")?.to_string(),
                bar_size: arg_string(args, "bar_size")?.to_string(),
                outside_regular_trading_hours: false,
            };
            Ok(
                serde_json::to_value(backend.historical_bars(&request).await?)
                    .map_err(output_error)?,
            )
        }
        "ibkr_orders_list" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(serde_json::to_value(backend.orders(&account).await?).map_err(output_error)?)
        }
        "ibkr_order_status" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(serde_json::to_value(
                backend
                    .order_status(&account, arg_string(args, "broker_order_id")?)
                    .await?,
            )
            .map_err(output_error)?)
        }
        "ibkr_executions_list" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(serde_json::to_value(backend.executions(&account).await?).map_err(output_error)?)
        }
        "ibkr_audit_tail" => {
            let limit = args
                .get("limit")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(100);
            Ok(serde_json::to_value(
                audit_writer
                    .tail_verified(AuditTailRequest::new(limit))
                    .await?,
            )
            .map_err(output_error)?)
        }
        _ => Err(unavailable_mcp_tool(name)),
    }
}

async fn record_mcp_tool_audit(
    audit_writer: &SqliteAuditWriter,
    tool_name: &str,
    scope: &str,
    result: &Result<Value, GatewayError>,
) -> Result<(), GatewayError> {
    let status = match result {
        Ok(_) => AuditResultStatus::Completed,
        Err(error) => mcp_audit_status_for_error(error),
    };
    let mut event = crate::internal::mcp::build_mcp_tool_event(tool_name, scope, status);
    if let Err(error) = result {
        event.error_code = Some(error.code);
    }
    audit_writer.append(&event).await
}

const fn mcp_audit_status_for_error(error: &GatewayError) -> AuditResultStatus {
    match error.code {
        ErrorCode::AuthMissingScope | ErrorCode::AuditReadForbidden => {
            AuditResultStatus::DeniedScope
        }
        ErrorCode::ReadonlyWriteForbidden
        | ErrorCode::ReadonlyOrderPreviewForbidden
        | ErrorCode::ReadonlyOrderSubmitForbidden
        | ErrorCode::ReadonlyOrderCancelForbidden
        | ErrorCode::OrderPreviewDisabled
        | ErrorCode::OrderPolicyRefused
        | ErrorCode::PaperTradingDisabled
        | ErrorCode::PaperApprovalRequired
        | ErrorCode::PaperIdempotencyConflict
        | ErrorCode::LiveTradingDisabled
        | ErrorCode::LiveGateMissing
        | ErrorCode::LiveLimitRefused
        | ErrorCode::LiveKillSwitchClosed
        | ErrorCode::LiveMigrationRequired => AuditResultStatus::Refused,
        _ => AuditResultStatus::Failed,
    }
}

fn unavailable_mcp_tool(name: &str) -> GatewayError {
    if crate::internal::mcp::is_forbidden_tool_name(name) {
        return crate::internal::mcp::refuse_forbidden_tool(name);
    }
    GatewayError::new(
        ErrorCode::ReadonlyWriteForbidden,
        format!("MCP tool {name} is not available on the production stdio transport"),
        false,
        Some("Use a registered read-only MCP tool".to_string()),
    )
}

fn arg_string<'a>(args: &'a Value, key: &str) -> Result<&'a str, GatewayError> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| invalid_mcp_request(&format!("Missing MCP argument: {key}")))
}

fn parse_contract_id(contract_id: &str) -> Result<ContractId, GatewayError> {
    ContractId::new(contract_id).ok_or_else(|| {
        GatewayError::new(
            ErrorCode::InputInvalidContract,
            "Contract id is required",
            false,
            Some("Use a resolved contract id".to_string()),
        )
    })
}

fn invalid_mcp_request(message: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::ConfigInvalid,
        message,
        false,
        Some("Send a valid MCP tools/call request".to_string()),
    )
}

fn output_error(_error: serde_json::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::OutputUnsafe,
        "Failed to serialize MCP tool result",
        false,
        Some("Retry the MCP request".to_string()),
    )
}

fn io_error(_error: std::io::Error) -> GatewayError {
    GatewayError::new(
        ErrorCode::BrokerBackendUnavailable,
        "MCP stdio transport failed",
        true,
        Some("Restart the MCP client connection".to_string()),
    )
}

fn jsonrpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn jsonrpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::audit::{AuditHmacKey, AuditTailRequest};
    use crate::internal::auth::{ACCOUNTS_READ, HEALTH_READ, ScopeSet};
    use crate::internal::backend::{FakeBackend, FakeFixtureStore};
    use crate::internal::domain::{ErrorCode, GatewayError};
    use std::sync::Arc;

    async fn test_writer() -> Result<SqliteAuditWriter, GatewayError> {
        let key = Arc::new(AuditHmacKey::ephemeral()?);
        SqliteAuditWriter::connect("sqlite::memory:", key).await
    }

    fn fake_backend() -> FakeBackend {
        FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"))
    }

    #[tokio::test]
    async fn tools_list_only_advertises_locally_enabled_scopes() -> Result<(), GatewayError> {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let scopes = ScopeSet::read_only([HEALTH_READ])?;
        let Some(response) = handle_stdio_request(
            &backend,
            &writer,
            &scopes,
            &json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list"
            }),
        )
        .await
        else {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                "tools/list did not produce a response",
                false,
                None,
            ));
        };

        let Some(tools) = response["result"]["tools"].as_array() else {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                "tools/list did not return an array",
                false,
                None,
            ));
        };
        let names = tools
            .iter()
            .map(|tool| tool["name"].as_str().unwrap_or_default())
            .collect::<Vec<_>>();

        assert!(names.contains(&"ibkr_health"));
        assert!(!names.contains(&"ibkr_accounts_list"));
        Ok(())
    }

    #[tokio::test]
    async fn tools_call_denies_missing_scope_before_backend_and_audits() -> Result<(), GatewayError>
    {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let scopes = ScopeSet::read_only([HEALTH_READ])?;
        let Some(response) = handle_stdio_request(
            &backend,
            &writer,
            &scopes,
            &json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "ibkr_accounts_list",
                    "arguments": {}
                }
            }),
        )
        .await
        else {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                "tools/call did not produce a response",
                false,
                None,
            ));
        };

        assert!(response.get("error").is_some());
        let tail = writer.tail_verified(AuditTailRequest::new(10)).await?;
        assert_eq!(tail.events.len(), 1);
        let event = &tail.events[0].event;
        assert_eq!(event.tool_name.as_deref(), Some("ibkr_accounts_list"));
        assert_eq!(event.scopes, vec![ACCOUNTS_READ.to_string()]);
        assert_eq!(event.error_code, Some(ErrorCode::AuthMissingScope));
        assert_eq!(
            event.event_type,
            crate::internal::audit::AuditEventType::ToolDeniedScope
        );
        Ok(())
    }

    #[tokio::test]
    async fn initialized_notification_has_no_stdio_response() -> Result<(), GatewayError> {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let scopes = ScopeSet::read_only([HEALTH_READ])?;
        let response = handle_stdio_request(
            &backend,
            &writer,
            &scopes,
            &json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }),
        )
        .await;

        assert!(response.is_none());
        Ok(())
    }
}
