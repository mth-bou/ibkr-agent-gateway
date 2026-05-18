//! Operator CLI library for tests and the `ibkr-agent` binary.

pub mod audit;
pub mod commands;
pub mod output;
pub mod runtime;

use crate::cli::runtime::CliRuntime;
use crate::internal::audit::{AuditDecision, AuditEventType, AuditResultStatus};
use crate::internal::auth::{
    ACCOUNTS_READ, AUDIT_READ, HEALTH_READ, MARKETDATA_READ, ORDERS_LIVE_CANCEL,
    ORDERS_LIVE_SUBMIT, ORDERS_PAPER_CANCEL, ORDERS_PAPER_SUBMIT, ORDERS_PREVIEW, ORDERS_READ,
    PORTFOLIO_READ, POSITIONS_READ,
};
use crate::internal::domain::{ErrorCode, GatewayError};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// IBKR Agent Gateway operator CLI.
#[derive(Debug, Parser)]
#[command(name = "ibkr-agent")]
#[command(about = "Local IBKR Agent Gateway CLI")]
pub struct Cli {
    /// Optional config path. The current CLI defaults to fake fixtures when omitted.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    /// Emit JSON output.
    #[arg(long, global = true)]
    pub json: bool,
    /// Optional request id for future correlation.
    #[arg(long, global = true)]
    pub request_id: Option<String>,
    /// Command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Live broker writer selected by operator CLI smoke commands.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum LiveBrokerChoice {
    /// Do not call a broker; return a deterministic local candidate id.
    LocalCandidate,
    /// Use the configured Client Portal Gateway live writer.
    ClientPortal,
    /// Refuse live writes explicitly.
    Refusing,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Gateway health.
    Health,
    /// Broker backend commands.
    Backend {
        /// Backend subcommand.
        #[command(subcommand)]
        command: BackendCommand,
    },
    /// Broker session commands.
    Session {
        /// Session subcommand.
        #[command(subcommand)]
        command: SessionCommand,
    },
    /// Account commands.
    Account {
        /// Account subcommand.
        #[command(subcommand)]
        command: AccountCommand,
    },
    /// Accounts commands.
    Accounts {
        /// Accounts subcommand.
        #[command(subcommand)]
        command: AccountsCommand,
    },
    /// Approval commands.
    Approvals {
        /// Approval subcommand.
        #[command(subcommand)]
        command: ApprovalsCommand,
    },
    /// Portfolio commands.
    Portfolio {
        /// Portfolio subcommand.
        #[command(subcommand)]
        command: PortfolioCommand,
    },
    /// Positions commands.
    Positions {
        /// Positions subcommand.
        #[command(subcommand)]
        command: PositionsCommand,
    },
    /// Contract commands.
    Contracts {
        /// Contract subcommand.
        #[command(subcommand)]
        command: ContractsCommand,
    },
    /// Market data commands.
    Market {
        /// Market subcommand.
        #[command(subcommand)]
        command: MarketCommand,
    },
    /// Order and trading workflow commands.
    Orders {
        /// Orders subcommand.
        #[command(subcommand)]
        command: OrdersCommand,
    },
    /// Execution read-only commands.
    Executions {
        /// Executions subcommand.
        #[command(subcommand)]
        command: ExecutionsCommand,
    },
    /// Audit commands.
    Audit {
        /// Audit subcommand.
        #[command(subcommand)]
        command: AuditCommand,
    },
    /// MCP commands.
    Mcp {
        /// MCP subcommand.
        #[command(subcommand)]
        command: McpCommand,
    },
    /// Sidecar relay commands.
    Sidecar {
        /// Sidecar subcommand.
        #[command(subcommand)]
        command: SidecarCommand,
    },
}

/// Backend commands.
#[derive(Debug, Subcommand)]
pub enum BackendCommand {
    /// Show backend status.
    Status,
}

/// Session commands.
#[derive(Debug, Subcommand)]
pub enum SessionCommand {
    /// Show safe manual session requirements.
    Requirements,
}

/// Account commands.
#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    /// Show account summary.
    Summary {
        /// Account id.
        #[arg(long)]
        account: String,
    },
}

/// Accounts commands.
#[derive(Debug, Subcommand)]
pub enum AccountsCommand {
    /// List accessible accounts.
    List,
}

/// Approval commands.
#[derive(Debug, Subcommand)]
pub enum ApprovalsCommand {
    /// Create a local paper approval record.
    Create {
        /// Account id.
        #[arg(long)]
        account: String,
        /// Preview id returned by `orders preview`.
        #[arg(long)]
        preview_id: String,
        /// Approval TTL in seconds.
        #[arg(long, default_value_t = 300)]
        ttl_seconds: i64,
    },
}

/// Portfolio commands.
#[derive(Debug, Subcommand)]
pub enum PortfolioCommand {
    /// Show portfolio snapshot.
    Snapshot {
        /// Account id.
        #[arg(long)]
        account: String,
    },
}

/// Positions commands.
#[derive(Debug, Subcommand)]
pub enum PositionsCommand {
    /// List positions.
    List {
        /// Account id.
        #[arg(long)]
        account: String,
    },
}

/// Contract commands.
#[derive(Debug, Subcommand)]
pub enum ContractsCommand {
    /// Search contracts.
    Search {
        /// Query text.
        query: String,
        /// Asset class, accepted for contract compatibility.
        #[arg(long)]
        asset_class: Option<String>,
        /// Currency, accepted for contract compatibility.
        #[arg(long)]
        currency: Option<String>,
        /// Exchange, accepted for contract compatibility.
        #[arg(long)]
        exchange: Option<String>,
    },
    /// Resolve a contract.
    Resolve {
        /// Symbol or query.
        query: String,
        /// Asset class, accepted for contract compatibility.
        #[arg(long)]
        asset_class: Option<String>,
        /// Currency, accepted for contract compatibility.
        #[arg(long)]
        currency: Option<String>,
        /// Exchange, accepted for contract compatibility.
        #[arg(long)]
        exchange: Option<String>,
    },
}

/// Market data commands.
#[derive(Debug, Subcommand)]
pub enum MarketCommand {
    /// Show market snapshot.
    Snapshot {
        /// Contract id.
        #[arg(long)]
        contract_id: String,
    },
    /// Show historical bars.
    Bars {
        /// Contract id.
        #[arg(long)]
        contract_id: String,
        /// Duration.
        #[arg(long)]
        duration: String,
        /// Bar size.
        #[arg(long)]
        bar_size: String,
    },
}

/// Orders commands.
#[derive(Debug, Subcommand)]
pub enum OrdersCommand {
    /// List read-only orders.
    List {
        /// Account id.
        #[arg(long)]
        account: String,
    },
    /// Read order status.
    Status {
        /// Account id.
        #[arg(long)]
        account: String,
        /// Broker order id.
        #[arg(long)]
        broker_order_id: String,
    },
    /// Create a non-executable order preview.
    Preview {
        /// Account id.
        #[arg(long)]
        account: String,
        /// Symbol.
        #[arg(long)]
        symbol: String,
        /// Side.
        #[arg(long)]
        side: String,
        /// Quantity.
        #[arg(long)]
        quantity: String,
        /// Limit price.
        #[arg(long)]
        limit_price: String,
        /// Currency.
        #[arg(long, default_value = "USD")]
        currency: String,
        /// Explicitly enable preview for this local command.
        #[arg(long, default_value_t = false)]
        enable_preview: bool,
    },
    /// Submit a paper order candidate when explicitly enabled.
    Submit {
        /// Account id.
        #[arg(long)]
        account: Option<String>,
        /// Approval id returned by `approvals create`.
        #[arg(long)]
        approval_id: Option<String>,
        /// Idempotency key.
        #[arg(long)]
        idempotency_key: Option<String>,
        /// Explicitly enable paper submit.
        #[arg(long, default_value_t = false)]
        enable_paper: bool,
    },
    /// Cancel a paper order candidate when explicitly enabled.
    Cancel {
        /// Account id.
        #[arg(long)]
        account: Option<String>,
        /// Broker order id.
        #[arg(long)]
        broker_order_id: Option<String>,
        /// Idempotency key.
        #[arg(long)]
        idempotency_key: Option<String>,
        /// Explicitly enable paper cancel.
        #[arg(long, default_value_t = false)]
        enable_paper: bool,
    },
    /// Live order submit gated by explicit live flags.
    LiveSubmit {
        /// Account id.
        #[arg(long)]
        account: String,
        /// Approval id returned by `approvals create`.
        #[arg(long)]
        approval_id: String,
        /// Idempotency key.
        #[arg(long)]
        idempotency_key: String,
        /// Explicitly enable live submit.
        #[arg(long, default_value_t = false)]
        enable_live: bool,
        /// Simulate the required live submit scope.
        #[arg(long, default_value_t = false)]
        live_scope: bool,
        /// Simulate an open kill switch.
        #[arg(long, default_value_t = false)]
        open_kill_switch: bool,
        /// Acknowledge the paper-to-live checklist.
        #[arg(long, default_value_t = false)]
        acknowledge_paper_to_live: bool,
        /// Live writer backend.
        #[arg(long, value_enum, default_value_t = LiveBrokerChoice::LocalCandidate)]
        live_broker: LiveBrokerChoice,
    },
    /// Live order cancel gated by explicit live flags.
    LiveCancel {
        /// Account id.
        #[arg(long)]
        account: String,
        /// Broker order id.
        #[arg(long)]
        broker_order_id: String,
        /// Idempotency key.
        #[arg(long)]
        idempotency_key: String,
        /// Explicitly enable live cancel.
        #[arg(long, default_value_t = false)]
        enable_live: bool,
        /// Simulate the required live cancel scope.
        #[arg(long, default_value_t = false)]
        live_scope: bool,
        /// Simulate an open kill switch.
        #[arg(long, default_value_t = false)]
        open_kill_switch: bool,
        /// Acknowledge the paper-to-live checklist.
        #[arg(long, default_value_t = false)]
        acknowledge_paper_to_live: bool,
        /// Live writer backend.
        #[arg(long, value_enum, default_value_t = LiveBrokerChoice::LocalCandidate)]
        live_broker: LiveBrokerChoice,
    },
    /// Forbidden order modify.
    Modify,
    /// Forbidden order approve.
    Approve,
}

/// Executions commands.
#[derive(Debug, Subcommand)]
pub enum ExecutionsCommand {
    /// List executions.
    List {
        /// Account id.
        #[arg(long)]
        account: String,
        /// Optional from timestamp.
        #[arg(long)]
        from: Option<String>,
    },
}

/// Audit commands.
#[derive(Debug, Subcommand)]
pub enum AuditCommand {
    /// Tail recent audit events.
    Tail {
        /// Maximum number of events.
        #[arg(long, default_value_t = 100)]
        limit: u32,
        /// SQLite database URL.
        #[arg(long, default_value = "")]
        database_url: String,
    },
    /// Export recent audit events as redacted JSONL.
    Export {
        /// Maximum number of events.
        #[arg(long, default_value_t = 500)]
        limit: u32,
        /// SQLite database URL.
        #[arg(long, default_value = "")]
        database_url: String,
    },
    /// Verify the full audit HMAC chain.
    Verify {
        /// SQLite database URL.
        #[arg(long, default_value = "")]
        database_url: String,
        /// Environment variable containing the audit HMAC key for external DBs.
        #[arg(long, default_value = "")]
        hmac_secret_env: String,
    },
}

/// MCP commands.
#[derive(Debug, Subcommand)]
pub enum McpCommand {
    /// Serve MCP locally.
    Serve {
        /// Transport.
        #[arg(long)]
        transport: String,
        /// Describe the selected MCP transport and exit without serving.
        #[arg(long, default_value_t = false)]
        describe: bool,
        /// Explicitly enable remote MCP HTTP for this invocation.
        #[arg(long, default_value_t = false)]
        enable_remote_mcp: bool,
        /// HTTP bind address for remote MCP.
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: String,
    },
}

/// Sidecar commands.
#[derive(Debug, Subcommand)]
pub enum SidecarCommand {
    /// Sidecar identity commands.
    Identity {
        /// Identity subcommand.
        #[command(subcommand)]
        command: SidecarIdentityCommand,
    },
    /// Sidecar pairing commands.
    Pairing {
        /// Pairing subcommand.
        #[command(subcommand)]
        command: SidecarPairingCommand,
    },
}

/// Sidecar identity commands.
#[derive(Debug, Subcommand)]
pub enum SidecarIdentityCommand {
    /// Create a sidecar identity.
    Create {
        /// Display name.
        #[arg(long)]
        display_name: Option<String>,
        /// Public key or public key fingerprint.
        #[arg(long)]
        public_key: String,
    },
}

/// Sidecar pairing commands.
#[derive(Debug, Subcommand)]
pub enum SidecarPairingCommand {
    /// Create an explicit sidecar pairing record.
    Create {
        /// Remote gateway instance id.
        #[arg(long)]
        remote_instance_id: String,
        /// Sidecar id.
        #[arg(long)]
        sidecar_id: String,
        /// User id.
        #[arg(long)]
        user_id: String,
        /// Pairing TTL in seconds.
        #[arg(long, default_value_t = 300)]
        ttl_seconds: i64,
    },
    /// Revoke a sidecar pairing record.
    Revoke {
        /// Pairing id.
        #[arg(long)]
        pairing_id: String,
    },
}

/// Parses command line args and runs the CLI.
pub async fn run_from_args(
    args: impl IntoIterator<Item = impl Into<std::ffi::OsString> + Clone>,
) -> Result<(), crate::internal::domain::GatewayError> {
    let cli = Cli::parse_from(args);
    run(cli).await
}

/// Runs the parsed CLI.
pub async fn run(cli: Cli) -> Result<(), crate::internal::domain::GatewayError> {
    let runtime = CliRuntime::load(cli.config.as_deref()).await?;
    let audit_metadata = command_audit_metadata(&cli.command);
    if let Some(metadata) = audit_metadata
        && !runtime.scopes.contains(metadata.scope)
    {
        let error = GatewayError::new(
            ErrorCode::AuthMissingScope,
            format!("Missing required scope: {}", metadata.scope),
            false,
            Some("Enable the required local scope in config".to_string()),
        );
        let result = Err(error);
        let audit_event_id = record_command_audit(&runtime, &metadata, &result).await?;
        return attach_audit_event_id(result, audit_event_id);
    }

    let result = match &cli.command {
        Command::Health => commands::health::run(cli.json),
        Command::Backend {
            command: BackendCommand::Status,
        } => commands::backend::status(runtime.backend.as_ref(), cli.json).await,
        Command::Session {
            command: SessionCommand::Requirements,
        } => commands::backend::requirements(runtime.backend.as_ref(), cli.json).await,
        Command::Accounts {
            command: AccountsCommand::List,
        } => commands::accounts::list(runtime.backend.as_ref(), cli.json).await,
        Command::Approvals {
            command:
                ApprovalsCommand::Create {
                    account,
                    preview_id,
                    ttl_seconds,
                },
        } => {
            commands::approvals::create(
                &runtime.audit_writer,
                account,
                preview_id,
                *ttl_seconds,
                cli.json,
            )
            .await
        }
        Command::Account {
            command: AccountCommand::Summary { account },
        } => commands::account::summary(runtime.backend.as_ref(), account, cli.json).await,
        Command::Portfolio {
            command: PortfolioCommand::Snapshot { account },
        } => commands::portfolio::snapshot(runtime.backend.as_ref(), account, cli.json).await,
        Command::Positions {
            command: PositionsCommand::List { account },
        } => commands::positions::list(runtime.backend.as_ref(), account, cli.json).await,
        Command::Contracts {
            command:
                ContractsCommand::Search {
                    query,
                    asset_class: _,
                    currency: _,
                    exchange: _,
                },
        } => commands::contracts::search(runtime.backend.as_ref(), query, cli.json).await,
        Command::Contracts {
            command:
                ContractsCommand::Resolve {
                    query,
                    asset_class: _,
                    currency: _,
                    exchange: _,
                },
        } => commands::contracts::resolve(runtime.backend.as_ref(), query, cli.json).await,
        Command::Market {
            command: MarketCommand::Snapshot { contract_id },
        } => commands::market::snapshot(runtime.backend.as_ref(), contract_id, cli.json).await,
        Command::Market {
            command:
                MarketCommand::Bars {
                    contract_id,
                    duration,
                    bar_size,
                },
        } => {
            commands::market::bars(
                runtime.backend.as_ref(),
                contract_id,
                duration,
                bar_size,
                cli.json,
            )
            .await
        }
        Command::Orders {
            command: OrdersCommand::List { account },
        } => commands::orders::list(runtime.backend.as_ref(), account, cli.json).await,
        Command::Orders {
            command:
                OrdersCommand::Status {
                    account,
                    broker_order_id,
                },
        } => {
            commands::orders::status(runtime.backend.as_ref(), account, broker_order_id, cli.json)
                .await
        }
        Command::Orders {
            command:
                OrdersCommand::Preview {
                    account,
                    symbol,
                    side,
                    quantity,
                    limit_price,
                    currency,
                    enable_preview,
                },
        } => {
            commands::orders_preview::preview(
                &runtime.audit_writer,
                runtime.backend.as_ref(),
                commands::orders_preview::PreviewRequest {
                    account,
                    symbol,
                    side,
                    quantity,
                    limit_price,
                    currency,
                    enable_preview: *enable_preview,
                },
                cli.json,
            )
            .await
        }
        Command::Orders {
            command:
                OrdersCommand::Submit {
                    account,
                    approval_id,
                    idempotency_key,
                    enable_paper,
                },
        } => match (
            account.as_deref(),
            approval_id.as_deref(),
            idempotency_key.as_deref(),
        ) {
            (Some(account), Some(approval_id), Some(idempotency_key)) => {
                commands::orders_paper::submit(
                    &runtime.audit_writer,
                    account,
                    approval_id,
                    idempotency_key,
                    *enable_paper,
                    cli.json,
                )
                .await
            }
            (_, None, _) => Err(GatewayError::new(
                ErrorCode::PaperApprovalRequired,
                "Paper submit requires an approval id",
                false,
                Some("Run `ibkr-agent approvals create` and pass --approval-id".to_string()),
            )),
            _ => commands::orders::refuse_write("submit"),
        },
        Command::Orders {
            command:
                OrdersCommand::Cancel {
                    account,
                    broker_order_id,
                    idempotency_key,
                    enable_paper,
                },
        } => match (
            account.as_deref(),
            broker_order_id.as_deref(),
            idempotency_key.as_deref(),
        ) {
            (Some(account), Some(broker_order_id), Some(idempotency_key)) => {
                commands::orders_paper::cancel(
                    &runtime.audit_writer,
                    account,
                    broker_order_id,
                    idempotency_key,
                    *enable_paper,
                    cli.json,
                )
                .await
            }
            _ => commands::orders::refuse_write("cancel"),
        },
        Command::Orders {
            command:
                OrdersCommand::LiveSubmit {
                    account,
                    approval_id,
                    idempotency_key,
                    enable_live,
                    live_scope,
                    open_kill_switch,
                    acknowledge_paper_to_live,
                    live_broker,
                },
        } => {
            let writer = runtime.live_order_writer(*live_broker)?;
            commands::orders_live::submit(
                commands::orders_live::LiveOrderCommandRuntime {
                    audit_writer: &runtime.audit_writer,
                    backend: runtime.backend.as_ref(),
                    writer: writer.as_ref(),
                    live_config: &runtime.live_trading_config,
                },
                account,
                approval_id,
                idempotency_key,
                commands::orders_live::LiveCommandGates {
                    enable_live: *enable_live,
                    live_scope: *live_scope,
                    open_kill_switch: *open_kill_switch,
                    acknowledge_migration: *acknowledge_paper_to_live,
                },
                cli.json,
            )
            .await
        }
        Command::Orders {
            command:
                OrdersCommand::LiveCancel {
                    account,
                    broker_order_id,
                    idempotency_key,
                    enable_live,
                    live_scope,
                    open_kill_switch,
                    acknowledge_paper_to_live,
                    live_broker,
                },
        } => {
            let writer = runtime.live_order_writer(*live_broker)?;
            commands::orders_live::cancel(
                commands::orders_live::LiveOrderCommandRuntime {
                    audit_writer: &runtime.audit_writer,
                    backend: runtime.backend.as_ref(),
                    writer: writer.as_ref(),
                    live_config: &runtime.live_trading_config,
                },
                account,
                broker_order_id,
                idempotency_key,
                commands::orders_live::LiveCommandGates {
                    enable_live: *enable_live,
                    live_scope: *live_scope,
                    open_kill_switch: *open_kill_switch,
                    acknowledge_migration: *acknowledge_paper_to_live,
                },
                cli.json,
            )
            .await
        }
        Command::Orders {
            command: OrdersCommand::Modify,
        } => commands::orders::refuse_write("modify"),
        Command::Orders {
            command: OrdersCommand::Approve,
        } => commands::orders::refuse_write("approve"),
        Command::Executions {
            command: ExecutionsCommand::List { account, from: _ },
        } => commands::orders::executions(runtime.backend.as_ref(), account, cli.json).await,
        Command::Audit {
            command:
                AuditCommand::Tail {
                    limit,
                    database_url,
                },
        } => commands::audit::tail(&runtime.audit_writer, database_url, *limit, cli.json).await,
        Command::Audit {
            command:
                AuditCommand::Export {
                    limit,
                    database_url,
                },
        } => commands::audit::export(&runtime.audit_writer, database_url, *limit, cli.json).await,
        Command::Audit {
            command:
                AuditCommand::Verify {
                    database_url,
                    hmac_secret_env,
                },
        } => {
            commands::audit::verify(
                &runtime.audit_writer,
                database_url,
                hmac_secret_env,
                cli.json,
            )
            .await
        }
        Command::Mcp {
            command:
                McpCommand::Serve {
                    transport,
                    describe,
                    enable_remote_mcp,
                    bind,
                },
        } => {
            commands::mcp::serve(
                runtime.backend.as_ref(),
                &runtime.audit_writer,
                &runtime.scopes,
                commands::mcp::McpServeOptions {
                    transport,
                    describe: *describe,
                    enable_remote_mcp: *enable_remote_mcp,
                    bind,
                    remote_mcp_config: &runtime.remote_mcp_config,
                    json: cli.json,
                    live_reconciler_interval_seconds: runtime.live_reconciler_interval_seconds,
                },
            )
            .await
        }
        Command::Sidecar {
            command:
                SidecarCommand::Identity {
                    command:
                        SidecarIdentityCommand::Create {
                            display_name,
                            public_key,
                        },
                },
        } => commands::sidecar::identity_create(display_name.clone(), public_key, cli.json),
        Command::Sidecar {
            command:
                SidecarCommand::Pairing {
                    command:
                        SidecarPairingCommand::Create {
                            remote_instance_id,
                            sidecar_id,
                            user_id,
                            ttl_seconds,
                        },
                },
        } => commands::sidecar::pairing_create(
            remote_instance_id,
            sidecar_id,
            user_id,
            *ttl_seconds,
            cli.json,
        ),
        Command::Sidecar {
            command:
                SidecarCommand::Pairing {
                    command: SidecarPairingCommand::Revoke { pairing_id },
                },
        } => commands::sidecar::pairing_revoke(pairing_id, cli.json),
    };

    if let Some(metadata) = audit_metadata {
        let audit_event_id = record_command_audit(&runtime, &metadata, &result).await?;
        return attach_audit_event_id(result, audit_event_id);
    }

    result
}

#[derive(Clone, Copy)]
struct CommandAuditMetadata<'a> {
    tool_name: &'static str,
    scope: &'static str,
    event_type: AuditEventType,
    account: Option<&'a str>,
}

fn command_audit_metadata(command: &Command) -> Option<CommandAuditMetadata<'_>> {
    match command {
        Command::Health => Some(meta(
            "ibkr_health",
            HEALTH_READ,
            AuditEventType::ToolCompleted,
            None,
        )),
        Command::Backend { .. } => Some(meta(
            "ibkr_backend_status",
            HEALTH_READ,
            AuditEventType::BackendSessionChecked,
            None,
        )),
        Command::Session { .. } => Some(meta(
            "ibkr_session_requirements",
            HEALTH_READ,
            AuditEventType::BackendSessionChecked,
            None,
        )),
        Command::Accounts { .. } => Some(meta(
            "ibkr_accounts_list",
            ACCOUNTS_READ,
            AuditEventType::ToolCompleted,
            None,
        )),
        Command::Approvals {
            command: ApprovalsCommand::Create { account, .. },
        } => Some(meta(
            "ibkr_approval_create",
            ORDERS_PAPER_SUBMIT,
            AuditEventType::PaperApprovalRecorded,
            Some(account),
        )),
        Command::Account {
            command: AccountCommand::Summary { account },
        } => Some(meta(
            "ibkr_account_summary",
            PORTFOLIO_READ,
            AuditEventType::ToolCompleted,
            Some(account),
        )),
        Command::Portfolio {
            command: PortfolioCommand::Snapshot { account },
        } => Some(meta(
            "ibkr_portfolio_snapshot",
            PORTFOLIO_READ,
            AuditEventType::ToolCompleted,
            Some(account),
        )),
        Command::Positions {
            command: PositionsCommand::List { account },
        } => Some(meta(
            "ibkr_positions_list",
            POSITIONS_READ,
            AuditEventType::ToolCompleted,
            Some(account),
        )),
        Command::Contracts { .. } => Some(meta(
            "ibkr_contracts",
            MARKETDATA_READ,
            AuditEventType::ToolCompleted,
            None,
        )),
        Command::Market { .. } => Some(meta(
            "ibkr_marketdata",
            MARKETDATA_READ,
            AuditEventType::ToolCompleted,
            None,
        )),
        Command::Orders {
            command: OrdersCommand::List { account },
        } => Some(meta(
            "ibkr_orders_list",
            ORDERS_READ,
            AuditEventType::ToolCompleted,
            Some(account),
        )),
        Command::Orders {
            command:
                OrdersCommand::Status {
                    account,
                    broker_order_id: _,
                },
        } => Some(meta(
            "ibkr_order_status",
            ORDERS_READ,
            AuditEventType::ToolCompleted,
            Some(account),
        )),
        Command::Orders {
            command: OrdersCommand::Preview { account, .. },
        } => Some(meta(
            "ibkr_order_preview",
            ORDERS_PREVIEW,
            AuditEventType::OrderPreviewCreated,
            Some(account),
        )),
        Command::Orders {
            command: OrdersCommand::Submit { account, .. },
        } => Some(meta(
            "ibkr_paper_order_submit",
            ORDERS_PAPER_SUBMIT,
            AuditEventType::PaperOrderSubmitted,
            account.as_deref(),
        )),
        Command::Orders {
            command: OrdersCommand::Cancel { account, .. },
        } => Some(meta(
            "ibkr_paper_order_cancel",
            ORDERS_PAPER_CANCEL,
            AuditEventType::PaperOrderCancelled,
            account.as_deref(),
        )),
        Command::Orders {
            command: OrdersCommand::LiveSubmit { account, .. },
        } => Some(meta(
            "ibkr_live_order_submit",
            ORDERS_LIVE_SUBMIT,
            AuditEventType::LiveOrderLifecycleChanged,
            Some(account),
        )),
        Command::Orders {
            command: OrdersCommand::LiveCancel { account, .. },
        } => Some(meta(
            "ibkr_live_order_cancel",
            ORDERS_LIVE_CANCEL,
            AuditEventType::LiveOrderLifecycleChanged,
            Some(account),
        )),
        Command::Orders { .. } => Some(meta(
            "ibkr_order_write_refusal",
            ORDERS_READ,
            AuditEventType::ToolRefused,
            None,
        )),
        Command::Executions {
            command: ExecutionsCommand::List { account, .. },
        } => Some(meta(
            "ibkr_executions_list",
            ORDERS_READ,
            AuditEventType::ToolCompleted,
            Some(account),
        )),
        Command::Audit { .. } => Some(meta(
            "ibkr_audit",
            AUDIT_READ,
            AuditEventType::ToolCompleted,
            None,
        )),
        Command::Mcp { .. } => Some(meta(
            "ibkr_mcp_serve",
            HEALTH_READ,
            AuditEventType::ToolCompleted,
            None,
        )),
        Command::Sidecar { .. } => Some(meta(
            "ibkr_sidecar",
            HEALTH_READ,
            AuditEventType::ToolCompleted,
            None,
        )),
    }
}

const fn meta<'a>(
    tool_name: &'static str,
    scope: &'static str,
    event_type: AuditEventType,
    account: Option<&'a str>,
) -> CommandAuditMetadata<'a> {
    CommandAuditMetadata {
        tool_name,
        scope,
        event_type,
        account,
    }
}

async fn record_command_audit(
    runtime: &CliRuntime,
    metadata: &CommandAuditMetadata<'_>,
    result: &Result<(), GatewayError>,
) -> Result<crate::internal::domain::AuditEventId, GatewayError> {
    let mut event = audit::build_cli_audit_event(
        metadata.tool_name,
        metadata.scope,
        metadata.event_type,
        AuditResultStatus::Completed,
    );
    if let Some(account) = metadata.account
        && let Some(account_id) = crate::internal::domain::AccountId::new(account)
    {
        event.account_id_hash = Some(
            runtime
                .audit_hmac_key
                .compute_account_id_hash(account_id.as_str())?,
        );
    }
    if let Err(error) = result {
        event.error_code = Some(error.code);
        event.result_status = audit_result_status_for_error(error);
        event.decision = audit_decision_for_error(error);
        event.event_type = audit_event_type_for_error(metadata.event_type, error);
    }
    let audit_event_id = event.event_id.clone();
    runtime.audit_writer.append(&event).await?;
    Ok(audit_event_id)
}

fn attach_audit_event_id(
    result: Result<(), GatewayError>,
    audit_event_id: crate::internal::domain::AuditEventId,
) -> Result<(), GatewayError> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(error.with_audit_event_id(audit_event_id)),
    }
}

const fn audit_result_status_for_error(error: &GatewayError) -> AuditResultStatus {
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
        | ErrorCode::ApprovalPreviewMismatch
        | ErrorCode::ApprovalConsumed
        | ErrorCode::PaperIdempotencyConflict
        | ErrorCode::LiveTradingDisabled
        | ErrorCode::LiveGateMissing
        | ErrorCode::LiveLimitRefused
        | ErrorCode::LiveKillSwitchClosed
        | ErrorCode::LiveMigrationRequired => AuditResultStatus::Refused,
        _ => AuditResultStatus::Failed,
    }
}

const fn audit_decision_for_error(error: &GatewayError) -> AuditDecision {
    match error.code {
        ErrorCode::AuthMissingScope | ErrorCode::AuditReadForbidden => AuditDecision::Deny,
        ErrorCode::ReadonlyWriteForbidden
        | ErrorCode::ReadonlyOrderPreviewForbidden
        | ErrorCode::ReadonlyOrderSubmitForbidden
        | ErrorCode::ReadonlyOrderCancelForbidden
        | ErrorCode::OrderPreviewDisabled
        | ErrorCode::OrderPolicyRefused
        | ErrorCode::PaperTradingDisabled
        | ErrorCode::PaperApprovalRequired
        | ErrorCode::ApprovalPreviewMismatch
        | ErrorCode::ApprovalConsumed
        | ErrorCode::PaperIdempotencyConflict
        | ErrorCode::LiveTradingDisabled
        | ErrorCode::LiveGateMissing
        | ErrorCode::LiveLimitRefused
        | ErrorCode::LiveKillSwitchClosed
        | ErrorCode::LiveMigrationRequired => AuditDecision::Refuse,
        _ => AuditDecision::Allow,
    }
}

const fn audit_event_type_for_error(
    fallback: AuditEventType,
    error: &GatewayError,
) -> AuditEventType {
    match error.code {
        ErrorCode::AuthMissingScope | ErrorCode::AuditReadForbidden => {
            AuditEventType::ToolDeniedScope
        }
        ErrorCode::ReadonlyWriteForbidden
        | ErrorCode::ReadonlyOrderPreviewForbidden
        | ErrorCode::ReadonlyOrderSubmitForbidden
        | ErrorCode::ReadonlyOrderCancelForbidden
        | ErrorCode::OrderPreviewDisabled
        | ErrorCode::OrderPolicyRefused
        | ErrorCode::PaperTradingDisabled
        | ErrorCode::PaperApprovalRequired
        | ErrorCode::ApprovalPreviewMismatch
        | ErrorCode::ApprovalConsumed
        | ErrorCode::PaperIdempotencyConflict
        | ErrorCode::LiveTradingDisabled
        | ErrorCode::LiveGateMissing
        | ErrorCode::LiveLimitRefused
        | ErrorCode::LiveKillSwitchClosed
        | ErrorCode::LiveMigrationRequired => AuditEventType::ToolRefused,
        _ => fallback,
    }
}

/// Maps a gateway error to CLI process exit code.
#[must_use]
pub fn exit_code(error: &crate::internal::domain::GatewayError) -> i32 {
    match error.code {
        crate::internal::domain::ErrorCode::InputMissingAccount
        | crate::internal::domain::ErrorCode::InputUnauthorizedAccount
        | crate::internal::domain::ErrorCode::InputAmbiguousAccount
        | crate::internal::domain::ErrorCode::InputAmbiguousContract
        | crate::internal::domain::ErrorCode::InputUnsupportedAssetClass
        | crate::internal::domain::ErrorCode::InputInvalidContract
        | crate::internal::domain::ErrorCode::InputInvalidTimeRange
        | crate::internal::domain::ErrorCode::MarketDataStale
        | crate::internal::domain::ErrorCode::AuditChainInvalid => 2,
        crate::internal::domain::ErrorCode::BrokerSessionRequired
        | crate::internal::domain::ErrorCode::BrokerSessionExpired
        | crate::internal::domain::ErrorCode::BrokerBackendUnavailable => 3,
        crate::internal::domain::ErrorCode::AuthMissingScope
        | crate::internal::domain::ErrorCode::AuthTokenMissing
        | crate::internal::domain::ErrorCode::AuthTokenInvalid
        | crate::internal::domain::ErrorCode::AuthTokenExpired
        | crate::internal::domain::ErrorCode::AuthInvalidIssuer
        | crate::internal::domain::ErrorCode::AuthInvalidAudience
        | crate::internal::domain::ErrorCode::AuthScopeNotAllowedInMvp
        | crate::internal::domain::ErrorCode::AuditReadForbidden => 4,
        crate::internal::domain::ErrorCode::BrokerRateLimited
        | crate::internal::domain::ErrorCode::BrokerCapabilityUnavailable
        | crate::internal::domain::ErrorCode::BrokerResponseInvalid => 5,
        crate::internal::domain::ErrorCode::OutputUnsafe => 6,
        crate::internal::domain::ErrorCode::ConfigInvalid
        | crate::internal::domain::ErrorCode::ConfigMissingBrokerBaseUrl
        | crate::internal::domain::ErrorCode::ConfigTlsBypassNonLocalhost
        | crate::internal::domain::ErrorCode::ConfigWriteToolsForbidden
        | crate::internal::domain::ErrorCode::ConfigRemoteMcpForbidden
        | crate::internal::domain::ErrorCode::ConfigSidecarForbidden
        | crate::internal::domain::ErrorCode::ConfigLiveTradingForbidden
        | crate::internal::domain::ErrorCode::AuthLocalOnlyMvp => 7,
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{AUDIT_READ, AuditCommand, Command, command_audit_metadata};

    #[test]
    fn audit_commands_require_audit_read_scope() {
        let command = Command::Audit {
            command: AuditCommand::Tail {
                limit: 10,
                database_url: String::new(),
            },
        };
        let metadata = command_audit_metadata(&command);

        let Some(metadata) = metadata else {
            unreachable!("audit commands must be guarded by audit metadata");
        };
        assert_eq!(metadata.scope, AUDIT_READ);
    }
}
