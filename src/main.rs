use clap::{Parser, Subcommand};
use anyhow::Result;
use log::info;

mod commands;
mod cardano;
mod metrics;
mod utils;

use commands::*;

#[derive(Parser)]
#[command(name = "cardano-validator")]
#[command(about = "A CLI toolkit for Cardano stake pool operators")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path 
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Check stake pool status and health
    Status {
        /// Pool ID to check
        #[arg(short, long)]
        pool_id: Option<String>,
    },
    /// Generate rewards summary
    Rewards {
        /// Epoch to analyze
        #[arg(short, long)]
        epoch: Option<u64>,
        /// Output format (json, table, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Monitor pool metrics
    Monitor {
        /// Enable Prometheus metrics export
        #[arg(long)]
        prometheus: bool,
        /// Metrics export port
        #[arg(long, default_value = "9090")]
        port: u16,
    },
    /// Perform health checks
    Health {
        /// Check all configured pools
        #[arg(long)]
        all: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
    }

    info!("Starting Cardano Validator CLI Toolkit");

    // Load configuration
    let config = utils::config::load_config(&cli.config)?;

    match cli.command {
        Commands::Status { pool_id } => {
            pool_status::execute(pool_id, &config).await?;
        }
        Commands::Rewards { epoch, format } => {
            rewards::execute(epoch, &format, &config).await?;
        }
        Commands::Monitor { prometheus, port } => {
            monitoring::execute(prometheus, port, &config).await?;
        }
        Commands::Health { all } => {
            health_check::execute(all, &config).await?;
        }
    }

    Ok(())
}