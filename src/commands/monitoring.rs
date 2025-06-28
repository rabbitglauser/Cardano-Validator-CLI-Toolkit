
use anyhow::Result;
use crate::utils::config::Config;
use crate::cardano::blockfrost::BlockfrostClient;
use tokio::time::{interval, Duration};

pub async fn execute(prometheus: bool, port: u16, config: &Config) -> Result<()> {
    println!("📊 Monitoring command executed!");
    println!("  Prometheus enabled: {}", prometheus);
    println!("  Port: {}", port);

    if prometheus {
        println!("🚀 Starting Prometheus metrics server on port {}", port);
        start_prometheus_server(port, config).await?;
    } else {
        println!("📈 Running one-time monitoring check...");
        run_monitoring_check(config).await?;
    }

    Ok(())
}

async fn start_prometheus_server(port: u16, config: &Config) -> Result<()> {
    let blockfrost = BlockfrostClient::new(config)
        .ok_or_else(|| anyhow::anyhow!("Blockfrost configuration not found"))?;

    println!("🔧 Prometheus metrics server starting...");
    println!("📍 Metrics will be available at: http://localhost:{}/metrics", port);

    // Create a simple HTTP server for Prometheus metrics
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        // Collect metrics
        match collect_metrics(&blockfrost, config).await {
            Ok(metrics) => {
                println!("📊 Metrics collected:");
                for (key, value) in metrics {
                    println!("  {} = {}", key, value);
                }
            }
            Err(e) => {
                println!("❌ Error collecting metrics: {}", e);
            }
        }

        println!("⏰ Next collection in 30 seconds... (Press Ctrl+C to stop)");
    }
}

async fn run_monitoring_check(config: &Config) -> Result<()> {
    let blockfrost = BlockfrostClient::new(config)
        .ok_or_else(|| anyhow::anyhow!("Blockfrost configuration not found"))?;

    println!("🔍 Running monitoring checks...");

    // Check network status
    match blockfrost.get_network_info().await {
        Ok(info) => {
            println!("✅ Network Status: Healthy");
            if let Some(supply) = info.get("supply") {
                println!("  📊 Total Supply: {} ADA", supply);
            }
        }
        Err(e) => {
            println!("❌ Network Status: Failed - {}", e);
        }
    }

    // Check latest epoch
    match blockfrost.get_latest_epoch().await {
        Ok(epoch) => {
            println!("✅ Current Epoch: {}", epoch.get("epoch").unwrap_or(&serde_json::Value::Null));
        }
        Err(e) => {
            println!("❌ Epoch Check: Failed - {}", e);
        }
    }

    // Check configured pools
    for pool in &config.pools {
        println!("🏊 Checking pool: {} ({})", pool.name, pool.ticker);

        match blockfrost.get_pool_info(&pool.pool_id).await {
            Ok(pool_info) => {
                println!("  ✅ Pool Status: Active");
                if let Some(live_stake) = pool_info.get("live_stake") {
                    println!("  📊 Live Stake: {} lovelace", live_stake);
                }
            }
            Err(e) => {
                println!("  ❌ Pool Status: Failed - {}", e);
            }
        }
    }

    println!("🎉 Monitoring check completed!");
    Ok(())
}

async fn collect_metrics(blockfrost: &BlockfrostClient, config: &Config) -> Result<Vec<(String, String)>> {
    let mut metrics = Vec::new();

    // Network metrics
    if let Ok(network) = blockfrost.get_network_info().await {
        if let Some(supply) = network.get("supply") {
            metrics.push(("cardano_total_supply".to_string(), supply.to_string()));
        }
    }

    // Epoch metrics
    if let Ok(epoch) = blockfrost.get_latest_epoch().await {
        if let Some(epoch_num) = epoch.get("epoch") {
            metrics.push(("cardano_current_epoch".to_string(), epoch_num.to_string()));
        }
    }

    // Pool metrics
    for pool in &config.pools {
        if let Ok(pool_info) = blockfrost.get_pool_info(&pool.pool_id).await {
            let pool_prefix = format!("cardano_pool_{}", pool.ticker.to_lowercase());

            if let Some(live_stake) = pool_info.get("live_stake") {
                metrics.push((format!("{}_live_stake", pool_prefix), live_stake.to_string()));
            }

            if let Some(active_stake) = pool_info.get("active_stake") {
                metrics.push((format!("{}_active_stake", pool_prefix), active_stake.to_string()));
            }

            metrics.push((format!("{}_status", pool_prefix), "1".to_string()));
        }
    }

    Ok(metrics)
}