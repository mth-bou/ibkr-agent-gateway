//! Operator CLI library for tests and the `ibkr-agent` binary.

pub mod audit;
pub mod commands;
pub mod output;

use clap::{Parser, Subcommand};
use ibkr_backend::{FakeBackend, FakeFixtureStore};
use std::path::PathBuf;

/// IBKR Agent Gateway operator CLI.
#[derive(Debug, Parser)]
#[command(name = "ibkr-agent")]
#[command(about = "Local read-only IBKR Agent Gateway CLI")]
pub struct Cli {
    /// Optional config path. US1 defaults to fake fixtures when omitted.
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
    /// Order read-only commands.
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
    /// Forbidden order preview.
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
    /// Forbidden order submit.
    Submit {
        /// Account id.
        #[arg(long)]
        account: Option<String>,
        /// Idempotency key.
        #[arg(long)]
        idempotency_key: Option<String>,
        /// Explicitly enable paper submit.
        #[arg(long, default_value_t = false)]
        enable_paper: bool,
    },
    /// Forbidden order cancel.
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
        #[arg(long, default_value = "sqlite::memory:")]
        database_url: String,
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
        public_key: Option<String>,
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
) -> Result<(), ibkr_domain::GatewayError> {
    let cli = Cli::parse_from(args);
    run(cli).await
}

/// Runs the parsed CLI.
pub async fn run(cli: Cli) -> Result<(), ibkr_domain::GatewayError> {
    let _config_path = cli.config;
    let _request_id = cli.request_id;
    let backend = FakeBackend::new(FakeFixtureStore::new("tests/fixtures/cpapi"));

    match cli.command {
        Command::Health => commands::health::run(cli.json),
        Command::Backend {
            command: BackendCommand::Status,
        } => commands::backend::status(&backend, cli.json).await,
        Command::Session {
            command: SessionCommand::Requirements,
        } => commands::backend::requirements(&backend, cli.json).await,
        Command::Accounts {
            command: AccountsCommand::List,
        } => commands::accounts::list(&backend, cli.json).await,
        Command::Approvals {
            command:
                ApprovalsCommand::Create {
                    account,
                    ttl_seconds,
                },
        } => commands::approvals::create(&account, ttl_seconds, cli.json),
        Command::Account {
            command: AccountCommand::Summary { account },
        } => commands::account::summary(&backend, &account, cli.json).await,
        Command::Portfolio {
            command: PortfolioCommand::Snapshot { account },
        } => commands::portfolio::snapshot(&backend, &account, cli.json).await,
        Command::Positions {
            command: PositionsCommand::List { account },
        } => commands::positions::list(&backend, &account, cli.json).await,
        Command::Contracts {
            command:
                ContractsCommand::Search {
                    query,
                    asset_class: _,
                    currency: _,
                    exchange: _,
                },
        } => commands::contracts::search(&backend, &query, cli.json).await,
        Command::Contracts {
            command:
                ContractsCommand::Resolve {
                    query,
                    asset_class: _,
                    currency: _,
                    exchange: _,
                },
        } => commands::contracts::resolve(&backend, &query, cli.json).await,
        Command::Market {
            command: MarketCommand::Snapshot { contract_id },
        } => commands::market::snapshot(&backend, &contract_id, cli.json).await,
        Command::Market {
            command:
                MarketCommand::Bars {
                    contract_id,
                    duration,
                    bar_size,
                },
        } => commands::market::bars(&backend, &contract_id, &duration, &bar_size, cli.json).await,
        Command::Orders {
            command: OrdersCommand::List { account },
        } => commands::orders::list(&backend, &account, cli.json).await,
        Command::Orders {
            command:
                OrdersCommand::Status {
                    account,
                    broker_order_id,
                },
        } => commands::orders::status(&backend, &account, &broker_order_id, cli.json).await,
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
                &backend,
                commands::orders_preview::PreviewRequest {
                    account: &account,
                    symbol: &symbol,
                    side: &side,
                    quantity: &quantity,
                    limit_price: &limit_price,
                    currency: &currency,
                    enable_preview,
                },
                cli.json,
            )
            .await
        }
        Command::Orders {
            command:
                OrdersCommand::Submit {
                    account,
                    idempotency_key,
                    enable_paper,
                },
        } => match (account, idempotency_key) {
            (Some(account), Some(idempotency_key)) => {
                commands::orders_paper::submit(&account, &idempotency_key, enable_paper, cli.json)
            }
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
        } => match (account, broker_order_id, idempotency_key) {
            (Some(account), Some(broker_order_id), Some(idempotency_key)) => {
                commands::orders_paper::cancel(
                    &account,
                    &broker_order_id,
                    &idempotency_key,
                    enable_paper,
                    cli.json,
                )
            }
            _ => commands::orders::refuse_write("cancel"),
        },
        Command::Orders {
            command:
                OrdersCommand::LiveSubmit {
                    account,
                    idempotency_key,
                    enable_live,
                    live_scope,
                    open_kill_switch,
                    acknowledge_paper_to_live,
                },
        } => commands::orders_live::submit(
            &account,
            &idempotency_key,
            commands::orders_live::LiveCommandGates {
                enable_live,
                live_scope,
                open_kill_switch,
                acknowledge_migration: acknowledge_paper_to_live,
            },
            cli.json,
        ),
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
                },
        } => commands::orders_live::cancel(
            &account,
            &broker_order_id,
            &idempotency_key,
            commands::orders_live::LiveCommandGates {
                enable_live,
                live_scope,
                open_kill_switch,
                acknowledge_migration: acknowledge_paper_to_live,
            },
            cli.json,
        ),
        Command::Orders {
            command: OrdersCommand::Modify,
        } => commands::orders::refuse_write("modify"),
        Command::Orders {
            command: OrdersCommand::Approve,
        } => commands::orders::refuse_write("approve"),
        Command::Executions {
            command: ExecutionsCommand::List { account, from: _ },
        } => commands::orders::executions(&backend, &account, cli.json).await,
        Command::Audit {
            command:
                AuditCommand::Tail {
                    limit,
                    database_url,
                },
        } => commands::audit::tail(&database_url, limit, cli.json).await,
        Command::Mcp {
            command:
                McpCommand::Serve {
                    transport,
                    enable_remote_mcp,
                    bind,
                },
        } => commands::mcp::serve(&transport, enable_remote_mcp, &bind, cli.json),
        Command::Sidecar {
            command:
                SidecarCommand::Identity {
                    command:
                        SidecarIdentityCommand::Create {
                            display_name,
                            public_key,
                        },
                },
        } => commands::sidecar::identity_create(display_name, public_key, cli.json),
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
            &remote_instance_id,
            &sidecar_id,
            &user_id,
            ttl_seconds,
            cli.json,
        ),
        Command::Sidecar {
            command:
                SidecarCommand::Pairing {
                    command: SidecarPairingCommand::Revoke { pairing_id },
                },
        } => commands::sidecar::pairing_revoke(&pairing_id, cli.json),
    }
}

/// Maps a gateway error to CLI process exit code.
#[must_use]
pub fn exit_code(error: &ibkr_domain::GatewayError) -> i32 {
    match error.code {
        ibkr_domain::ErrorCode::InputMissingAccount
        | ibkr_domain::ErrorCode::InputUnauthorizedAccount
        | ibkr_domain::ErrorCode::InputAmbiguousAccount
        | ibkr_domain::ErrorCode::InputAmbiguousContract
        | ibkr_domain::ErrorCode::InputUnsupportedAssetClass
        | ibkr_domain::ErrorCode::InputInvalidContract
        | ibkr_domain::ErrorCode::InputInvalidTimeRange
        | ibkr_domain::ErrorCode::MarketDataStale => 2,
        ibkr_domain::ErrorCode::BrokerSessionRequired
        | ibkr_domain::ErrorCode::BrokerSessionExpired
        | ibkr_domain::ErrorCode::BrokerBackendUnavailable => 3,
        ibkr_domain::ErrorCode::AuthMissingScope
        | ibkr_domain::ErrorCode::AuthTokenMissing
        | ibkr_domain::ErrorCode::AuthTokenInvalid
        | ibkr_domain::ErrorCode::AuthTokenExpired
        | ibkr_domain::ErrorCode::AuthInvalidIssuer
        | ibkr_domain::ErrorCode::AuthInvalidAudience
        | ibkr_domain::ErrorCode::AuthScopeNotAllowedInMvp
        | ibkr_domain::ErrorCode::AuditReadForbidden => 4,
        ibkr_domain::ErrorCode::BrokerRateLimited
        | ibkr_domain::ErrorCode::BrokerCapabilityUnavailable
        | ibkr_domain::ErrorCode::BrokerResponseInvalid => 5,
        ibkr_domain::ErrorCode::OutputUnsafe => 6,
        ibkr_domain::ErrorCode::ConfigInvalid
        | ibkr_domain::ErrorCode::ConfigMissingBrokerBaseUrl
        | ibkr_domain::ErrorCode::ConfigTlsBypassNonLocalhost
        | ibkr_domain::ErrorCode::ConfigWriteToolsForbidden
        | ibkr_domain::ErrorCode::ConfigRemoteMcpForbidden
        | ibkr_domain::ErrorCode::ConfigSidecarForbidden
        | ibkr_domain::ErrorCode::ConfigLiveTradingForbidden
        | ibkr_domain::ErrorCode::AuthLocalOnlyMvp => 7,
        _ => 1,
    }
}
