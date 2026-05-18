//! MCP serve command.

use crate::cli::{commands::account::parse_account_id, output::print_output};
use crate::internal::audit::{AuditResultStatus, AuditTailRequest, SqliteAuditWriter};
use crate::internal::auth::ScopeSet;
use crate::internal::backend::IbkrBackend;
use crate::internal::config::{LiveTradingConfig, RemoteMcpConfig};
use crate::internal::domain::{ContractId, ErrorCode, GatewayError, HistoricalBarsRequest};
use crate::internal::orders::LiveOrderWriter;
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
    /// Whether the server-owned live kill switch starts open.
    pub open_live_kill_switch: bool,
}

/// Runs `ibkr-agent mcp serve`.
pub async fn serve(
    backend: &dyn IbkrBackend,
    audit_writer: &SqliteAuditWriter,
    scopes: &ScopeSet,
    live_config: &LiveTradingConfig,
    live_writer: &dyn LiveOrderWriter,
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
                    live_config,
                    live_writer,
                    options.live_reconciler_interval_seconds,
                    options.open_live_kill_switch,
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
    live_config: &LiveTradingConfig,
    live_writer: &dyn LiveOrderWriter,
    live_reconciler_interval_seconds: u64,
    open_live_kill_switch: bool,
) -> Result<(), GatewayError> {
    let stdin = tokio::io::stdin();
    let mut lines = tokio::io::BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();
    let runtime = StdioToolRuntime {
        backend,
        audit_writer,
        scopes,
        live_config,
        live_writer,
        open_live_kill_switch,
    };
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
                    Ok(request) => handle_stdio_request(&runtime, &request).await,
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

#[derive(Clone, Copy)]
struct StdioToolRuntime<'a> {
    backend: &'a dyn IbkrBackend,
    audit_writer: &'a SqliteAuditWriter,
    scopes: &'a ScopeSet,
    live_config: &'a LiveTradingConfig,
    live_writer: &'a dyn LiveOrderWriter,
    open_live_kill_switch: bool,
}

async fn handle_stdio_request(runtime: &StdioToolRuntime<'_>, request: &Value) -> Option<Value> {
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
                "tools": crate::internal::mcp::registry::local_tool_schemas_for_scopes(runtime.scopes)
                    .into_iter()
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
        "tools/call" => Some(match call_tool(runtime, request).await {
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
        }),
        _ => Some(jsonrpc_error(id, -32601, "Method not found")),
    }
}

async fn call_tool(runtime: &StdioToolRuntime<'_>, request: &Value) -> Result<Value, GatewayError> {
    let params = request.get("params").unwrap_or(&Value::Null);
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_mcp_request("Tool name is required"))?;
    let args = params.get("arguments").unwrap_or(&Value::Null);

    let tool = crate::internal::mcp::registry::find_local_tool_schema(name)
        .ok_or_else(|| unavailable_mcp_tool(name))?;
    if let Err(error) = crate::internal::mcp::enforce_scope(runtime.scopes, &tool.scope) {
        record_mcp_tool_audit(runtime.audit_writer, name, &tool.scope, &Err(error.clone())).await?;
        return Err(error);
    }

    let result = execute_tool(runtime, name, args).await;
    record_mcp_tool_audit(runtime.audit_writer, name, &tool.scope, &result).await?;
    result
}

async fn execute_tool(
    runtime: &StdioToolRuntime<'_>,
    name: &str,
    args: &Value,
) -> Result<Value, GatewayError> {
    match name {
        "ibkr_health" => Ok(json!({ "gateway": "ok", "read_only": true })),
        "ibkr_backend_status" | "ibkr_session_requirements" => Ok(serde_json::to_value(
            runtime.backend.session_status().await?,
        )
        .map_err(output_error)?),
        "ibkr_accounts_list" => {
            Ok(serde_json::to_value(runtime.backend.list_accounts().await?)
                .map_err(output_error)?)
        }
        "ibkr_account_summary" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(runtime.backend.account_summary(&account).await?)
        }
        "ibkr_positions_list" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(
                serde_json::to_value(runtime.backend.positions(&account).await?)
                    .map_err(output_error)?,
            )
        }
        "ibkr_portfolio_snapshot" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(runtime.backend.portfolio_snapshot(&account).await?)
        }
        "ibkr_contracts_search" => Ok(serde_json::to_value(
            runtime
                .backend
                .search_contracts(arg_string(args, "query")?)
                .await?,
        )
        .map_err(output_error)?),
        "ibkr_contract_resolve" => Ok(serde_json::to_value(
            runtime
                .backend
                .resolve_contract(arg_string(args, "symbol")?)
                .await?,
        )
        .map_err(output_error)?),
        "ibkr_market_snapshot" => {
            let contract_id = parse_contract_id(arg_string(args, "contract_id")?)?;
            Ok(
                serde_json::to_value(runtime.backend.market_snapshot(&contract_id).await?)
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
                serde_json::to_value(runtime.backend.historical_bars(&request).await?)
                    .map_err(output_error)?,
            )
        }
        "ibkr_orders_list" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(
                serde_json::to_value(runtime.backend.orders(&account).await?)
                    .map_err(output_error)?,
            )
        }
        "ibkr_order_status" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(serde_json::to_value(
                runtime
                    .backend
                    .order_status(&account, arg_string(args, "broker_order_id")?)
                    .await?,
            )
            .map_err(output_error)?)
        }
        "ibkr_executions_list" => {
            let account = parse_account_id(arg_string(args, "account_id")?)?;
            Ok(
                serde_json::to_value(runtime.backend.executions(&account).await?)
                    .map_err(output_error)?,
            )
        }
        "ibkr_audit_tail" => {
            let limit = args
                .get("limit")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(100);
            Ok(serde_json::to_value(
                runtime
                    .audit_writer
                    .tail_verified(AuditTailRequest::new(limit))
                    .await?,
            )
            .map_err(output_error)?)
        }
        "ibkr_order_preview" | "ibkr_paper_order_submit" | "ibkr_paper_order_cancel" => {
            let context = crate::internal::mcp::order_workflows::McpOrderWorkflowContext {
                backend: runtime.backend,
                audit_writer: runtime.audit_writer,
            };
            crate::internal::mcp::order_workflows::handle_order_workflow_tool(
                &context,
                runtime.scopes,
                name,
                args,
            )
            .await
        }
        "ibkr_live_order_submit" | "ibkr_live_order_cancel" => {
            let live_policy =
                crate::cli::commands::orders_live::live_limit_policy(runtime.live_config)?;
            let policy_registry = crate::internal::risk::StaticPolicyRegistry::single(live_policy);
            let Some(currency) = crate::internal::domain::CurrencyCode::new("USD") else {
                return Err(GatewayError::new(
                    ErrorCode::OrderValidationFailed,
                    "Static currency is invalid",
                    false,
                    None,
                ));
            };
            let live_limit_context = crate::internal::risk::LiveLimitContext {
                symbol: "AAPL".to_string(),
                asset_class: crate::internal::domain::AssetClass::Stock,
                submitted_in_window: 0,
                submitted_in_session: 0,
                session_notional: Some(crate::internal::domain::Money {
                    amount: rust_decimal::Decimal::ZERO,
                    currency,
                }),
                market_snapshot: None,
            };
            let context = crate::internal::mcp::live_orders::McpLiveOrderContext {
                backend: runtime.backend,
                audit_writer: runtime.audit_writer,
                writer: runtime.live_writer,
                policy_registry: &policy_registry,
                live_config: runtime.live_config.clone(),
                live_limit_context,
                kill_switch: crate::cli::commands::orders_live::kill_switch(
                    runtime.open_live_kill_switch,
                ),
                migration_checklist: crate::cli::commands::orders_live::migration_checklist(
                    runtime.live_config.paper_to_live_checklist_acknowledged,
                ),
            };
            crate::internal::mcp::live_orders::handle_live_order_tool(
                &context,
                runtime.scopes,
                name,
                args,
            )
            .await
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
    use crate::internal::approval::ApprovalService;
    use crate::internal::audit::{AuditHmacKey, AuditTailRequest};
    use crate::internal::auth::{
        ACCOUNTS_READ, HEALTH_READ, ORDERS_PAPER_SUBMIT, ORDERS_PREVIEW, ScopeSet,
    };
    use crate::internal::backend::{FakeBackend, FakeFixtureStore};
    use crate::internal::config::LiveTradingConfig;
    use crate::internal::domain::{ErrorCode, GatewayError, LocalUserId, OrderPreviewId};
    use crate::internal::orders::LocalCandidateLiveWriter;
    use std::sync::Arc;

    async fn test_writer() -> Result<SqliteAuditWriter, GatewayError> {
        let key = Arc::new(AuditHmacKey::ephemeral()?);
        SqliteAuditWriter::connect("sqlite::memory:", key).await
    }

    fn fake_backend() -> FakeBackend {
        FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"))
    }

    async fn stdio_request(
        backend: &FakeBackend,
        writer: &SqliteAuditWriter,
        scopes: &ScopeSet,
        request: &Value,
    ) -> Option<Value> {
        let live_writer = LocalCandidateLiveWriter;
        let live_config = LiveTradingConfig::default();
        let runtime = StdioToolRuntime {
            backend,
            audit_writer: writer,
            scopes,
            live_config: &live_config,
            live_writer: &live_writer,
            open_live_kill_switch: false,
        };
        handle_stdio_request(&runtime, request).await
    }

    fn mcp_text_payload(response: &Value) -> Result<Value, GatewayError> {
        let Some(text) = response["result"]["content"][0]["text"].as_str() else {
            return Err(GatewayError::new(
                ErrorCode::OutputUnsafe,
                "MCP response did not include text content",
                false,
                None,
            ));
        };
        serde_json::from_str(text).map_err(|_| {
            GatewayError::new(
                ErrorCode::OutputUnsafe,
                "MCP response text was not valid JSON",
                false,
                None,
            )
        })
    }

    #[tokio::test]
    async fn tools_list_only_advertises_locally_enabled_scopes() -> Result<(), GatewayError> {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let scopes = ScopeSet::read_only([HEALTH_READ])?;
        let Some(response) = stdio_request(
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
        assert!(!names.contains(&"ibkr_order_preview"));
        Ok(())
    }

    #[tokio::test]
    async fn tools_list_advertises_preview_when_scope_is_enabled() -> Result<(), GatewayError> {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let scopes = ScopeSet::local_with_preview([HEALTH_READ, ORDERS_PREVIEW])?;
        let Some(response) = stdio_request(
            &backend,
            &writer,
            &scopes,
            &json!({
                "jsonrpc": "2.0",
                "id": 11,
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
        assert!(names.contains(&"ibkr_order_preview"));
        Ok(())
    }

    #[tokio::test]
    async fn tools_call_routes_order_preview_and_paper_submit() -> Result<(), GatewayError> {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let scopes =
            ScopeSet::local_with_paper([HEALTH_READ, ORDERS_PREVIEW, ORDERS_PAPER_SUBMIT])?;
        let Some(preview_response) = stdio_request(
            &backend,
            &writer,
            &scopes,
            &json!({
                "jsonrpc": "2.0",
                "id": 12,
                "method": "tools/call",
                "params": {
                    "name": "ibkr_order_preview",
                    "arguments": {
                        "account_id": "DU1234567",
                        "symbol": "AAPL",
                        "side": "buy",
                        "quantity": "1",
                        "order_type": "limit",
                        "limit_price": "100",
                        "time_in_force": "day"
                    }
                }
            }),
        )
        .await
        else {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                "tools/call did not produce a preview response",
                false,
                None,
            ));
        };
        let preview_payload = mcp_text_payload(&preview_response)?;
        let Some(preview_id) = preview_payload["preview_id"].as_str() else {
            return Err(GatewayError::new(
                ErrorCode::OutputUnsafe,
                "preview response did not include preview_id",
                false,
                None,
            ));
        };
        let preview_id = OrderPreviewId::parse(preview_id)?;
        let account_id = parse_account_id("DU1234567")?;
        let mut approval_service = ApprovalService::default();
        let approval = approval_service.create_approval(
            preview_id,
            account_id,
            LocalUserId::from_static("mcp-test"),
            300,
        );
        writer.append_approval(&approval).await?;

        let Some(submit_response) = stdio_request(
            &backend,
            &writer,
            &scopes,
            &json!({
                "jsonrpc": "2.0",
                "id": 13,
                "method": "tools/call",
                "params": {
                    "name": "ibkr_paper_order_submit",
                    "arguments": {
                        "account_id": "DU1234567",
                        "approval_id": approval.approval_id.as_uuid().to_string(),
                        "idempotency_key": "mcp-paper-submit-key"
                    }
                }
            }),
        )
        .await
        else {
            return Err(GatewayError::new(
                ErrorCode::ConfigInvalid,
                "tools/call did not produce a paper submit response",
                false,
                None,
            ));
        };
        let submit_payload = mcp_text_payload(&submit_response)?;
        assert_eq!(submit_payload["broker_order_id"], "paper-order-local");
        Ok(())
    }

    #[tokio::test]
    async fn tools_call_denies_missing_scope_before_backend_and_audits() -> Result<(), GatewayError>
    {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let scopes = ScopeSet::read_only([HEALTH_READ])?;
        let Some(response) = stdio_request(
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
        let response = stdio_request(
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
