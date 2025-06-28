use anyhow::Result;
use clap::{Parser, Subcommand};

mod cardano;
mod commands;
mod utils;

use utils::config::Config;

#[derive(Parser)]
#[command(name = "cardano-validator-cli")]
#[command(about = "A comprehensive CLI toolkit for Cardano stake pool operators")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive setup wizard for first-time configuration
    Setup,

    /// Check health status of your stake pool
    HealthCheck {
        /// Run continuous monitoring
        #[arg(long)]
        continuous: bool,

        /// Check interval in seconds for continuous mode
        #[arg(long, default_value = "30")]
        interval: u64,

        /// Export metrics to Prometheus format
        #[arg(long)]
        export: bool,
    },

    /// Monitor pool performance and metrics
    Monitor {
        /// Pool ID to monitor
        #[arg(short, long)]
        pool_id: Option<String>,

        /// Output format: table, json, csv
        #[arg(long, default_value = "table")]
        format: String,

        /// Run in continuous mode
        #[arg(long)]
        continuous: bool,

        /// Enable Prometheus metrics
        #[arg(long)]
        prometheus: bool,

        /// Prometheus port
        #[arg(long, default_value = "9090")]
        port: u16,
    },

    /// Check detailed pool status and information
    PoolStatus {
        /// Pool ID to check
        #[arg(short, long)]
        pool_id: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Compare with other pools
        #[arg(long)]
        compare: bool,
    },

    /// Calculate and analyze rewards
    Rewards {
        /// Epoch number to analyze
        #[arg(short, long)]
        epoch: Option<u64>,

        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,
    },

    /// Test API connection and configuration
    TestApi,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => {
            commands::setup::execute().await
        }
        _ => {
            // For all other commands, load config
            let config = Config::load_or_create_default()?;

            match cli.command {
                Commands::Setup => unreachable!(),
                Commands::HealthCheck { continuous, interval: _, export: _ } => {
                    commands::health_check::execute(continuous, &config).await
                }
                Commands::Monitor { pool_id: _, format: _, continuous: _, prometheus, port } => {
                    commands::monitoring::execute(prometheus, port, &config).await
                }
                Commands::PoolStatus { pool_id, detailed: _, compare: _ } => {
                    commands::pool_status::execute(pool_id, &config).await
                }
                Commands::Rewards { epoch, detailed } => {
                    commands::rewards::execute(epoch, detailed, &config).await
                }
                Commands::TestApi => {
                    commands::test_api::execute(&config).await
                }
            }
        }
    }
}