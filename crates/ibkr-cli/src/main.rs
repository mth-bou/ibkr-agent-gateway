//! Operator CLI entrypoint for `ibkr-agent`.

use clap::Parser;

#[tokio::main]
async fn main() {
    let cli = ibkr_cli::Cli::parse();
    if let Err(error) = ibkr_cli::run(cli).await {
        let json = std::env::args().any(|arg| arg == "--json");
        ibkr_cli::output::print_error(&error, json);
        std::process::exit(ibkr_cli::exit_code(&error));
    }
}
