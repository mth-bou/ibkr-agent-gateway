//! Operator CLI entrypoint for `ibkr-agent`.

use clap::Parser;
use ibkr_agent_gateway::cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(error) = ibkr_agent_gateway::cli::run(cli).await {
        let json = std::env::args().any(|arg| arg == "--json");
        ibkr_agent_gateway::cli::output::print_error(&error, json);
        std::process::exit(ibkr_agent_gateway::cli::exit_code(&error));
    }
}
