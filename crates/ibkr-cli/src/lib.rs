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
    Accounts {
        /// Accounts subcommand.
        #[command(subcommand)]
        command: AccountsCommand,
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

/// Accounts commands.
#[derive(Debug, Subcommand)]
pub enum AccountsCommand {
    /// List accessible accounts.
    List,
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
