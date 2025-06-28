use anyhow::Result;
use crate::cardano::cli::CardanoCli;
use crate::cardano::blockfrost::BlockfrostClient;
use crate::utils::config::Config;
use colored::*;

pub async fn execute(continuous: bool, config: &Config) -> Result<()> {
    run_health_check(config, continuous, 30, false).await
}

pub async fn run_health_check(config: &Config, continuous: bool, interval: u64, export: bool) -> Result<()> {
    println!("{}", "🏥 Starting Health Check...".bright_green().bold());

    loop {
        let health_status = perform_health_check(config).await?;

        display_health_status(&health_status);

        if export {
            export_prometheus_metrics(&health_status)?;
        }

        if !continuous {
            break;
        }

        println!("\n⏰ Waiting {} seconds for next check...", interval);
        tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
    }

    Ok(())
}

#[derive(Debug)]
struct HealthStatus {
    node_connected: bool,
    node_synced: bool,
    sync_progress: f64,
    current_epoch: u64,
    current_slot: u64,
    blockfrost_connected: bool,
    pool_active: bool,
    pool_id: Option<String>,
}

async fn perform_health_check(config: &Config) -> Result<HealthStatus> {
    let mut status = HealthStatus {
        node_connected: false,
        node_synced: false,
        sync_progress: 0.0,
        current_epoch: 0,
        current_slot: 0,
        blockfrost_connected: false,
        pool_active: false,
        pool_id: None,
    };

    // Check Cardano Node Connection
    let cli = CardanoCli::new(config);

    match cli.query_tip().await {
        Ok(tip) => {
            status.node_connected = true;
            status.sync_progress = tip["syncProgress"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            status.node_synced = status.sync_progress >= 99.9;
            status.current_epoch = tip["epoch"].as_u64().unwrap_or(0);
            status.current_slot = tip["slot"].as_u64().unwrap_or(0);

            println!("✅ Node connection: {}", "OK".green());
            println!("📊 Sync progress: {:.2}%", status.sync_progress);
            println!("📅 Current epoch: {}", status.current_epoch);
            println!("🎰 Current slot: {}", status.current_slot);
        },
        Err(e) => {
            println!("❌ Node connection: {} - {}", "FAILED".red(), e);
        }
    }

    // Check Blockfrost Connection
    if let Some(blockfrost_client) = BlockfrostClient::new(config) {
        match blockfrost_client.get_network_info().await {
            Ok(_) => {
                status.blockfrost_connected = true;
                println!("✅ Blockfrost API: {}", "OK".green());
            },
            Err(e) => {
                println!("❌ Blockfrost API: {} - {}", "FAILED".red(), e);
            }
        }
    }

    // Check Pool Status (if configured)
    if !config.pools.is_empty() {
        let pool = &config.pools[0]; // Check first pool
        status.pool_id = Some(pool.pool_id.clone());

        if status.node_connected {
            match cli.query_pool_params(&pool.pool_id).await {
                Ok(_) => {
                    status.pool_active = true;
                    println!("✅ Pool {} ({}): {}", pool.ticker, pool.pool_id, "ACTIVE".green());
                },
                Err(_) => {
                    println!("❌ Pool {} ({}): {}", pool.ticker, pool.pool_id, "INACTIVE".red());
                }
            }
        }
    }

    Ok(status)
}

fn display_health_status(status: &HealthStatus) {
    println!("\n{}", "📋 HEALTH CHECK SUMMARY".bright_blue().bold());
    println!("{}", "═".repeat(50).blue());

    let overall_health = if status.node_connected && status.node_synced && status.blockfrost_connected {
        "🟢 HEALTHY".green().bold()
    } else if status.node_connected || status.blockfrost_connected {
        "🟡 PARTIAL".yellow().bold()
    } else {
        "🔴 UNHEALTHY".red().bold()
    };

    println!("Overall Status: {}", overall_health);

    println!("\n{}", "🔍 Component Status:".bright_white().bold());
    println!("  Node Connection: {}", if status.node_connected { "✅" } else { "❌" });
    println!("  Node Synced: {}", if status.node_synced { "✅" } else { "❌" });
    println!("  Blockfrost API: {}", if status.blockfrost_connected { "✅" } else { "❌" });
    if let Some(_pool_id) = &status.pool_id {
        println!("  Pool Active: {}", if status.pool_active { "✅" } else { "❌" });
    }

    println!("{}", "═".repeat(50).blue());
}

fn export_prometheus_metrics(status: &HealthStatus) -> Result<()> {
    let metrics = format!(
        "# HELP cardano_node_connected Whether the Cardano node is connected\n\
         # TYPE cardano_node_connected gauge\n\
         cardano_node_connected {}\n\
         # HELP cardano_node_sync_progress Node synchronization progress\n\
         # TYPE cardano_node_sync_progress gauge\n\
         cardano_node_sync_progress {}\n\
         # HELP cardano_current_epoch Current epoch number\n\
         # TYPE cardano_current_epoch gauge\n\
         cardano_current_epoch {}\n\
         # HELP blockfrost_connected Whether Blockfrost API is connected\n\
         # TYPE blockfrost_connected gauge\n\
         blockfrost_connected {}\n",
        if status.node_connected { 1 } else { 0 },
        status.sync_progress / 100.0,
        status.current_epoch,
        if status.blockfrost_connected { 1 } else { 0 }
    );

    std::fs::write("cardano_health_metrics.prom", metrics)?;
    println!("📊 Metrics exported to: cardano_health_metrics.prom");

    Ok(())
}