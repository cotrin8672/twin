mod cli;
mod config;
mod core;
mod environment;
mod git;
mod hooks;
mod symlink;
mod tui;
mod utils;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::cli::commands::*;
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
            handle_create(args).await?;
        }
        Commands::List(args) => {
            handle_list(args).await?;
        }
        Commands::Remove(args) => {
            handle_remove(args).await?;
        }
        Commands::Config(args) => {
            handle_config(args).await?;
        }
        Commands::Tui => {
            todo!("Implement TUI")
        }
    }

    Ok(())
}
