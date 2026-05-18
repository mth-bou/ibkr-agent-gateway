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
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::task::Poll;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, MissedTickBehavior};

const MAX_HTTP_HEADER_BYTES: usize = 16 * 1024;
const MAX_HTTP_BODY_BYTES: usize = 1024 * 1024;
type HttpConnectionFuture<'a> = Pin<Box<dyn Future<Output = Result<(), GatewayError>> + 'a>>;

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
            if options.describe {
                crate::internal::mcp::serve_http_description(&config)?
            } else {
                return serve_http(
                    backend,
                    audit_writer,
                    live_config,
                    live_writer,
                    options.open_live_kill_switch,
                    &config,
                )
                .await;
            }
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

async fn serve_http(
    backend: &dyn IbkrBackend,
    audit_writer: &SqliteAuditWriter,
    live_config: &LiveTradingConfig,
    live_writer: &dyn LiveOrderWriter,
    open_live_kill_switch: bool,
    config: &RemoteMcpConfig,
) -> Result<(), GatewayError> {
    crate::internal::config::validate_remote_mcp_config(config, config.enabled)?;
    let Some(jwks_url) = config.jwks_url.clone() else {
        return Err(GatewayError::new(
            ErrorCode::ConfigInvalid,
            "Remote MCP JWKS URL is not configured",
            false,
            Some("Configure remote_mcp.jwks_url".to_string()),
        ));
    };
    let listener = TcpListener::bind(&config.bind_address).await.map_err(|_| {
        GatewayError::new(
            ErrorCode::BrokerBackendUnavailable,
            format!(
                "Unable to bind remote MCP HTTP listener on {}",
                config.bind_address
            ),
            true,
            Some("Choose an available remote_mcp bind address".to_string()),
        )
    })?;
    let jwks_client = crate::internal::oauth::JwksHttpClient::default();
    let jwks_cache = tokio::sync::Mutex::new(crate::internal::oauth::JwksCache::default());
    let rate_limiter = crate::internal::mcp::http_server::HttpMcpRateLimiter::from_config(config);
    let runtime = HttpServeRuntime {
        backend,
        audit_writer,
        live_config,
        live_writer,
        open_live_kill_switch,
        config,
        jwks_url,
        jwks_client,
        rate_limiter,
    };
    let mut connections: Vec<HttpConnectionFuture<'_>> = Vec::new();

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (mut stream, _) = accepted.map_err(io_error)?;
                if http_connection_limit_reached(config, connections.len()) {
                    write_http_response(&mut stream, http_connection_limit_response(config)).await?;
                } else {
                    connections.push(Box::pin(handle_http_connection(&runtime, &jwks_cache, stream)));
                }
            }
            result = poll_one_http_connection(&mut connections), if !connections.is_empty() => {
                result?;
            }
        }
    }
}

struct HttpServeRuntime<'a> {
    backend: &'a dyn IbkrBackend,
    audit_writer: &'a SqliteAuditWriter,
    live_config: &'a LiveTradingConfig,
    live_writer: &'a dyn LiveOrderWriter,
    open_live_kill_switch: bool,
    config: &'a RemoteMcpConfig,
    jwks_url: url::Url,
    jwks_client: crate::internal::oauth::JwksHttpClient,
    rate_limiter: crate::internal::mcp::http_server::HttpMcpRateLimiter,
}

async fn handle_http_request(
    runtime: &HttpServeRuntime<'_>,
    jwks_cache: &tokio::sync::Mutex<crate::internal::oauth::JwksCache>,
    request: ParsedHttpRequest,
) -> HttpResponse {
    if request.method == "GET"
        && request.path == crate::internal::mcp::oauth_metadata::PROTECTED_RESOURCE_METADATA_PATH
    {
        return http_transport_response(
            crate::internal::mcp::http_server::protected_resource_metadata_response(runtime.config),
        );
    }
    if request.method != "POST" || request.path != crate::internal::mcp::http_server::MCP_HTTP_PATH
    {
        return http_json_response(
            404,
            json!({
                "error": "not_found",
                "message": "Use POST /mcp or GET /.well-known/oauth-protected-resource"
            }),
        );
    }
    if !runtime.rate_limiter.allows(&request.headers) {
        return http_json_response(
            429,
            json!({
                "error": "rate_limited",
                "message": "Too many remote MCP authorization attempts"
            }),
        );
    }

    let body = match serde_json::from_slice::<Value>(&request.body) {
        Ok(body) => body,
        Err(_) => {
            return http_json_response(400, jsonrpc_error(Value::Null, -32700, "Parse error"));
        }
    };
    let jwks = match jwks_cache
        .lock()
        .await
        .get_or_fetch(
            &runtime.jwks_client,
            &runtime.jwks_url,
            time::OffsetDateTime::now_utc(),
        )
        .await
    {
        Ok(jwks) => jwks,
        Err(error) => {
            return http_json_response(
                503,
                json!({
                    "error": error.code,
                    "message": "Remote MCP cannot fetch JWKS"
                }),
            );
        }
    };
    let auth_verifier =
        match crate::internal::mcp::http_auth::RemoteMcpAuthVerifier::new(runtime.config, &jwks) {
            Ok(auth_verifier) => auth_verifier,
            Err(error) => {
                return http_json_response(
                    503,
                    json!({
                        "error": error.code,
                        "message": "Remote MCP cannot prepare token validation"
                    }),
                );
            }
        };
    let auth_context =
        match authorize_http_jsonrpc(runtime.config, &auth_verifier, &request.headers, &body) {
            Ok(context) => context,
            Err(response) => return http_transport_response(response),
        };
    let tool_runtime = StdioToolRuntime {
        backend: runtime.backend,
        audit_writer: runtime.audit_writer,
        scopes: &auth_context.scopes,
        live_config: runtime.live_config,
        live_writer: runtime.live_writer,
        open_live_kill_switch: runtime.open_live_kill_switch,
    };
    match handle_stdio_request(&tool_runtime, &body).await {
        Some(response) => http_json_response(200, response),
        None => http_json_response(202, json!({ "status": "accepted" })),
    }
}

async fn handle_http_connection(
    runtime: &HttpServeRuntime<'_>,
    jwks_cache: &tokio::sync::Mutex<crate::internal::oauth::JwksCache>,
    mut stream: TcpStream,
) -> Result<(), GatewayError> {
    let response = match read_http_request(&mut stream).await {
        Ok(request) => handle_http_request(runtime, jwks_cache, request).await,
        Err(error) => http_json_response(
            400,
            json!({
                "error": error.code,
                "message": error.message
            }),
        ),
    };
    write_http_response(&mut stream, response).await
}

fn http_connection_limit_reached(config: &RemoteMcpConfig, active_connections: usize) -> bool {
    active_connections >= config.max_connections
}

fn http_connection_limit_response(config: &RemoteMcpConfig) -> HttpResponse {
    http_json_response(
        503,
        json!({
            "error": "connection_limit_reached",
            "message": "Too many concurrent remote MCP HTTP connections",
            "max_connections": config.max_connections
        }),
    )
}

async fn poll_one_http_connection(
    connections: &mut Vec<HttpConnectionFuture<'_>>,
) -> Result<(), GatewayError> {
    std::future::poll_fn(|cx| {
        let mut index = 0;
        while index < connections.len() {
            match connections[index].as_mut().poll(cx) {
                Poll::Ready(result) => {
                    drop(connections.swap_remove(index));
                    return Poll::Ready(result);
                }
                Poll::Pending => index += 1,
            }
        }
        Poll::Pending
    })
    .await
}

fn authorize_http_jsonrpc(
    config: &RemoteMcpConfig,
    auth_verifier: &crate::internal::mcp::http_auth::RemoteMcpAuthVerifier,
    headers: &BTreeMap<String, String>,
    body: &Value,
) -> Result<
    crate::internal::auth::RemoteAuthContext,
    crate::internal::mcp::http_server::HttpMcpResponse,
> {
    let method = body
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if method == "tools/call" {
        let tool_name = body
            .get("params")
            .and_then(|params| params.get("name"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(tool) = crate::internal::mcp::registry::find_local_tool_schema(tool_name) else {
            return crate::internal::mcp::http_auth::authorize_remote_request_without_required_scope(
                config,
                auth_verifier,
                headers,
            );
        };
        return crate::internal::mcp::http_auth::authorize_remote_request_with_verifier(
            config,
            auth_verifier,
            headers,
            &tool.scope,
        );
    }
    crate::internal::mcp::http_auth::authorize_remote_request_without_required_scope(
        config,
        auth_verifier,
        headers,
    )
}

#[derive(Debug)]
struct ParsedHttpRequest {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

async fn read_http_request(stream: &mut TcpStream) -> Result<ParsedHttpRequest, GatewayError> {
    let mut buffer = Vec::new();
    let header_end = loop {
        let mut chunk = [0_u8; 1024];
        let read = stream.read(&mut chunk).await.map_err(io_error)?;
        if read == 0 {
            return Err(invalid_http_request("HTTP request ended before headers"));
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.len() > MAX_HTTP_HEADER_BYTES + MAX_HTTP_BODY_BYTES {
            return Err(invalid_http_request("HTTP request is too large"));
        }
        if let Some(index) = find_header_end(&buffer) {
            break index;
        }
        if buffer.len() > MAX_HTTP_HEADER_BYTES {
            return Err(invalid_http_request("HTTP request headers are too large"));
        }
    };
    let header_bytes = &buffer[..header_end];
    let header_text = std::str::from_utf8(header_bytes)
        .map_err(|_| invalid_http_request("HTTP request headers are not UTF-8"))?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines
        .next()
        .filter(|line| !line.trim().is_empty())
        .ok_or_else(|| invalid_http_request("HTTP request line is missing"))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| invalid_http_request("HTTP method is missing"))?
        .to_string();
    let path = request_parts
        .next()
        .ok_or_else(|| invalid_http_request("HTTP path is missing"))?
        .split('?')
        .next()
        .unwrap_or_default()
        .to_string();
    let mut headers = BTreeMap::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(invalid_http_request("HTTP header is malformed"));
        };
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
    }
    let content_length = headers
        .get("content-length")
        .map(|value| usize::from_str(value))
        .transpose()
        .map_err(|_| invalid_http_request("HTTP content-length is invalid"))?
        .unwrap_or(0);
    if content_length > MAX_HTTP_BODY_BYTES {
        return Err(invalid_http_request("HTTP request body is too large"));
    }

    let body_start = header_end + 4;
    let mut body = buffer.get(body_start..).unwrap_or_default().to_vec();
    while body.len() < content_length {
        let remaining = content_length - body.len();
        let mut chunk = vec![0_u8; remaining.min(8192)];
        let read = stream.read(&mut chunk).await.map_err(io_error)?;
        if read == 0 {
            return Err(invalid_http_request("HTTP request body ended early"));
        }
        body.extend_from_slice(&chunk[..read]);
    }
    body.truncate(content_length);

    Ok(ParsedHttpRequest {
        method,
        path,
        headers,
        body,
    })
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn http_transport_response(
    response: crate::internal::mcp::http_server::HttpMcpResponse,
) -> HttpResponse {
    let body = serde_json::to_vec(&response.body).unwrap_or_else(|_| b"{}".to_vec());
    HttpResponse {
        status: response.status,
        headers: response.headers,
        body,
    }
}

fn http_json_response(status: u16, body: Value) -> HttpResponse {
    let mut headers = BTreeMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    HttpResponse {
        status,
        headers,
        body: serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec()),
    }
}

async fn write_http_response(
    stream: &mut TcpStream,
    mut response: HttpResponse,
) -> Result<(), GatewayError> {
    response.headers.insert(
        "content-length".to_string(),
        response.body.len().to_string(),
    );
    response
        .headers
        .insert("connection".to_string(), "close".to_string());
    let mut head = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status,
        http_reason(response.status)
    );
    for (name, value) in &response.headers {
        head.push_str(name);
        head.push_str(": ");
        head.push_str(value);
        head.push_str("\r\n");
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes()).await.map_err(io_error)?;
    stream.write_all(&response.body).await.map_err(io_error)?;
    stream.flush().await.map_err(io_error)
}

const fn http_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "OK",
    }
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
            let context = crate::internal::mcp::live_orders::McpLiveOrderContext {
                backend: runtime.backend,
                audit_writer: runtime.audit_writer,
                writer: runtime.live_writer,
                policy_registry: &policy_registry,
                live_config: runtime.live_config.clone(),
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
        format!("MCP tool {name} is not available on this MCP transport"),
        false,
        Some("Use a registered MCP tool with the required scope".to_string()),
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

fn invalid_http_request(message: &str) -> GatewayError {
    GatewayError::new(
        ErrorCode::ConfigInvalid,
        message,
        false,
        Some("Send a valid HTTP MCP request".to_string()),
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

    #[tokio::test]
    async fn http_transport_applies_rate_limit_before_routing() -> Result<(), GatewayError> {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let live_writer = LocalCandidateLiveWriter;
        let live_config = LiveTradingConfig::default();
        let jwks_url =
            url::Url::parse("https://auth.example.com/.well-known/jwks.json").map_err(|_| {
                GatewayError::new(
                    ErrorCode::ConfigInvalid,
                    "test JWKS URL did not parse",
                    false,
                    None,
                )
            })?;
        let config = RemoteMcpConfig {
            jwks_url: Some(jwks_url.clone()),
            ..RemoteMcpConfig::default()
        };
        let runtime = HttpServeRuntime {
            backend: &backend,
            audit_writer: &writer,
            live_config: &live_config,
            live_writer: &live_writer,
            open_live_kill_switch: false,
            config: &config,
            jwks_url,
            jwks_client: crate::internal::oauth::JwksHttpClient::default(),
            rate_limiter: crate::internal::mcp::http_server::HttpMcpRateLimiter::from_config(
                &config,
            ),
        };
        let jwks_cache = tokio::sync::Mutex::new(crate::internal::oauth::JwksCache::default());
        let headers = std::collections::BTreeMap::from([(
            "x-forwarded-for".to_string(),
            "203.0.113.10".to_string(),
        )]);

        for _ in 0..120 {
            let response = handle_http_request(
                &runtime,
                &jwks_cache,
                ParsedHttpRequest {
                    method: "POST".to_string(),
                    path: crate::internal::mcp::http_server::MCP_HTTP_PATH.to_string(),
                    headers: headers.clone(),
                    body: b"not-json".to_vec(),
                },
            )
            .await;
            assert_eq!(response.status, 400);
        }

        let response = handle_http_request(
            &runtime,
            &jwks_cache,
            ParsedHttpRequest {
                method: "POST".to_string(),
                path: crate::internal::mcp::http_server::MCP_HTTP_PATH.to_string(),
                headers,
                body: b"not-json".to_vec(),
            },
        )
        .await;
        assert_eq!(response.status, 429);
        Ok(())
    }

    #[tokio::test]
    async fn http_transport_uses_configured_rate_limit() -> Result<(), GatewayError> {
        let writer = test_writer().await?;
        let backend = fake_backend();
        let live_writer = LocalCandidateLiveWriter;
        let live_config = LiveTradingConfig::default();
        let jwks_url =
            url::Url::parse("https://auth.example.com/.well-known/jwks.json").map_err(|_| {
                GatewayError::new(
                    ErrorCode::ConfigInvalid,
                    "test JWKS URL did not parse",
                    false,
                    None,
                )
            })?;
        let config = RemoteMcpConfig {
            jwks_url: Some(jwks_url.clone()),
            rate_limit_max_requests: 2,
            rate_limit_window_seconds: 60,
            ..RemoteMcpConfig::default()
        };
        let runtime = HttpServeRuntime {
            backend: &backend,
            audit_writer: &writer,
            live_config: &live_config,
            live_writer: &live_writer,
            open_live_kill_switch: false,
            config: &config,
            jwks_url,
            jwks_client: crate::internal::oauth::JwksHttpClient::default(),
            rate_limiter: crate::internal::mcp::http_server::HttpMcpRateLimiter::from_config(
                &config,
            ),
        };
        let jwks_cache = tokio::sync::Mutex::new(crate::internal::oauth::JwksCache::default());
        let headers = std::collections::BTreeMap::from([(
            "x-forwarded-for".to_string(),
            "203.0.113.20".to_string(),
        )]);

        for _ in 0..2 {
            let response = handle_http_request(
                &runtime,
                &jwks_cache,
                ParsedHttpRequest {
                    method: "POST".to_string(),
                    path: crate::internal::mcp::http_server::MCP_HTTP_PATH.to_string(),
                    headers: headers.clone(),
                    body: b"not-json".to_vec(),
                },
            )
            .await;
            assert_eq!(response.status, 400);
        }

        let response = handle_http_request(
            &runtime,
            &jwks_cache,
            ParsedHttpRequest {
                method: "POST".to_string(),
                path: crate::internal::mcp::http_server::MCP_HTTP_PATH.to_string(),
                headers,
                body: b"not-json".to_vec(),
            },
        )
        .await;
        assert_eq!(response.status, 429);
        Ok(())
    }

    #[test]
    fn http_connection_limit_rejects_when_active_count_reaches_configured_cap() {
        let config = RemoteMcpConfig {
            max_connections: 2,
            ..RemoteMcpConfig::default()
        };

        assert!(!http_connection_limit_reached(&config, 1));
        assert!(http_connection_limit_reached(&config, 2));

        let response = http_connection_limit_response(&config);
        assert_eq!(response.status, 503);
        let body: serde_json::Value =
            serde_json::from_slice(&response.body).unwrap_or(serde_json::Value::Null);
        assert_eq!(
            body.get("error").and_then(serde_json::Value::as_str),
            Some("connection_limit_reached")
        );
        assert_eq!(
            body.get("max_connections")
                .and_then(serde_json::Value::as_u64),
            Some(2)
        );
    }

    #[tokio::test]
    async fn http_connection_poller_does_not_wait_for_earlier_pending_connection()
    -> Result<(), GatewayError> {
        let mut connections: Vec<HttpConnectionFuture<'_>> = vec![
            Box::pin(async { std::future::pending::<Result<(), GatewayError>>().await }),
            Box::pin(async { Ok(()) }),
        ];

        tokio::time::timeout(
            std::time::Duration::from_millis(100),
            poll_one_http_connection(&mut connections),
        )
        .await
        .map_err(|_| {
            GatewayError::new(
                ErrorCode::BrokerBackendUnavailable,
                "HTTP connection poller waited on a pending connection",
                true,
                None,
            )
        })??;

        assert_eq!(connections.len(), 1);
        Ok(())
    }
}
