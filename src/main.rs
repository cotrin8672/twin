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
    // Only initialize if RUST_LOG is set or TWIN_VERBOSE/TWIN_DEBUG is set
    if std::env::var("RUST_LOG").is_ok()
        || std::env::var("TWIN_VERBOSE").is_ok()
        || std::env::var("TWIN_DEBUG").is_ok()
    {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    if std::env::var("TWIN_DEBUG").is_ok() {
                        "twin=debug".into()
                    } else if std::env::var("TWIN_VERBOSE").is_ok() {
                        "twin=info".into()
                    } else {
                        "twin=warn".into()
                    }
                }),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // Parse CLI arguments
    let cli = Cli::parse();

    // Handle commands
    match cli.command {
        Commands::Add(args) => {
            handle_add(args).await?;
        }
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
