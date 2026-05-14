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
    /// MCP commands.
    Mcp {
        /// MCP subcommand.
        #[command(subcommand)]
        command: McpCommand,
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
    Preview,
    /// Forbidden order submit.
    Submit,
    /// Forbidden order cancel.
    Cancel,
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

/// MCP commands.
#[derive(Debug, Subcommand)]
pub enum McpCommand {
    /// Serve MCP locally.
    Serve {
        /// Transport.
        #[arg(long)]
        transport: String,
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
            command: OrdersCommand::Preview,
        } => commands::orders::refuse_write("preview"),
        Command::Orders {
            command: OrdersCommand::Submit,
        } => commands::orders::refuse_write("submit"),
        Command::Orders {
            command: OrdersCommand::Cancel,
        } => commands::orders::refuse_write("cancel"),
        Command::Orders {
            command: OrdersCommand::Modify,
        } => commands::orders::refuse_write("modify"),
        Command::Orders {
            command: OrdersCommand::Approve,
        } => commands::orders::refuse_write("approve"),
        Command::Executions {
            command: ExecutionsCommand::List { account, from: _ },
        } => commands::orders::executions(&backend, &account, cli.json).await,
        Command::Mcp {
            command: McpCommand::Serve { transport },
        } => commands::mcp::serve(&transport, cli.json),
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
