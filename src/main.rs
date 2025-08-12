mod cli;
mod config;
mod core;
mod environment;
mod git;
mod symlink;
mod tui;
mod utils;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "twin=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle commands
    match cli.command {
        Commands::Create(args) => {
            todo!("Implement create command")
        }
        Commands::List(args) => {
            todo!("Implement list command")
        }
        Commands::Remove(args) => {
            todo!("Implement remove command")
        }
        Commands::Switch(args) => {
            todo!("Implement switch command")
        }
        Commands::Init(args) => {
            todo!("Implement init command")
        }
        Commands::Config(args) => {
            todo!("Implement config command")
        }
        Commands::Tui => {
            todo!("Implement TUI")
        }
    }
}
