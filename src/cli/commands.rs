use clap::{Parser, Subcommand};

/// ARMA command-line interface definition.
#[derive(Debug, Parser)]
#[command(name = "arma", version, about = "ARMA prompt guardrail runtime")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Supported lifecycle commands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    Start {
        #[arg(short = 'd', long = "daemon")]
        daemon: bool,
    },
    Stop,
    Restart {
        #[arg(short = 'd', long = "daemon")]
        daemon: bool,
    },
    Reload,
    Status,
    Manual,
}
