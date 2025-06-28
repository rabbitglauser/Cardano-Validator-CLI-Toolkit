use clap::{Parser, Subcommand};
use anyhow::Result;
use log::info;

mod cardano;
mod commands;
mod utils;

use utils::config::Config;

#[derive(Parser)]
#[command(name = "cardano-validator-cli")]
#[command(about = "A comprehensive CLI toolkit for Cardano stake pool management")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Health monitoring and alerts
    Health {
        /// Monitor all pools
        #[arg(short, long)]
        all: bool,
    },
    /// Advanced analytics and reporting
    Analytics {
        /// Specific pool ID to analyze
        #[arg(short, long)]
        pool_id: Option<String>,
        /// Number of epochs to analyze
        #[arg(short, long, default_value = "10")]
        epochs: u64,
        /// Generate detailed report
        #[arg(short, long)]
        detailed: bool,
        /// Export results to file
        #[arg(short = 'x', long)]  // Changed from -e to -x
        export: bool,
    },
    /// Rewards calculation and distribution
    Rewards {
        /// Target epoch for calculation
        #[arg(short, long)]
        epoch: Option<u64>,
        /// Export detailed breakdown
        #[arg(long)]
        detailed: bool,
    },
    /// Node management operations
    Node {
        /// Node operation
        #[command(subcommand)]
        operation: NodeOperation,
    },
}

#[derive(Subcommand)]
enum NodeOperation {
    /// Check node status
    Status,
    /// Start node service
    Start,
    /// Stop node service
    Stop,
    /// Restart node service
    Restart,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Cardano Validator CLI Toolkit");

    let cli = Cli::parse();
    let config = Config::load_or_create_default()?;

    match cli.command {
        Commands::Health { all } => {
            commands::health_check::execute(all, &config).await?;
        }
        Commands::Analytics { pool_id, epochs, detailed, export } => {
            commands::analytics::execute(pool_id, epochs, detailed, export, &config).await?;
        }
        Commands::Rewards { epoch, detailed } => {
            commands::rewards::execute(epoch, detailed, &config).await?;
        }
        Commands::Node { operation } => {
            match operation {
                NodeOperation::Status => commands::node::status(&config).await?,
                NodeOperation::Start => commands::node::start(&config).await?,
                NodeOperation::Stop => commands::node::stop(&config).await?,
                NodeOperation::Restart => commands::node::restart(&config).await?,
            }
        }
    }

    Ok(())
}