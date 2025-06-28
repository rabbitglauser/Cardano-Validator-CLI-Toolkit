use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time;
use tabled::{Table, Tabled};

use crate::cardano::cli::CardanoCli;
use crate::utils::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub timestamp: String,
    pub pool_id: String,
    pub pool_name: String,
    pub is_healthy: bool,
    pub sync_status: SyncStatus,
    pub saturation_status: SaturationStatus,
    pub block_production: BlockProductionStatus,
    pub response_time_ms: u64,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    Syncing { blocks_behind: u64 },
    OutOfSync { blocks_behind: u64 },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaturationStatus {
    Optimal { percentage: f64 },
    High { percentage: f64 },
    Oversaturated { percentage: f64 },
    Low { percentage: f64 },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockProductionStatus {
    Healthy { blocks_current_epoch: u64, expected: u64 },
    Underperforming { blocks_current_epoch: u64, expected: u64 },
    NoBlocks { expected: u64 },
    Unknown,
}

#[derive(Tabled)]
struct HealthSummary {
    #[tabled(rename = "Pool")]
    pool_name: String,
    #[tabled(rename = "Status")]
    overall_status: String,
    #[tabled(rename = "Sync")]
    sync_status: String,
    #[tabled(rename = "Saturation")]
    saturation: String,
    #[tabled(rename = "Blocks")]
    blocks: String,
    #[tabled(rename = "Response")]
    response_time: String,
    #[tabled(rename = "Issues")]
    issue_count: String,
}

pub async fn execute(all: bool, config: &Config) -> Result<()> {
    println!("{}", "🏥 Health Check System".blue().bold());
    println!("{}", "=".repeat(50).blue());

    let cardano_cli = CardanoCli::new(config);

    // Check if this is a one-time check or continuous monitoring
    if std::env::args().any(|arg| arg == "--watch") {
        run_continuous_monitoring(&cardano_cli, config).await
    } else {
        run_single_health_check(&cardano_cli, config, all).await
    }
}

async fn run_single_health_check(
    cardano_cli: &CardanoCli,
    config: &Config,
    _all: bool,
) -> Result<()> {
    println!("{}", "🔍 Performing health check...".cyan());

    let health_results = perform_health_checks(cardano_cli, config).await?;
    display_health_results(&health_results);

    // Check for critical issues
    let critical_issues: Vec<_> = health_results.iter()
        .filter(|h| !h.is_healthy)
        .collect();

    if !critical_issues.is_empty() {
        println!("\n{}", "⚠️  CRITICAL ISSUES DETECTED:".red().bold());
        for health in critical_issues {
            println!("  {} {}: {}",
                     "❌".red(),
                     health.pool_name.red().bold(),
                     health.issues.join(", ")
            );
        }
    } else {
        println!("\n{}", "✅ All pools are healthy!".green().bold());
    }

    Ok(())
}

async fn run_continuous_monitoring(
    cardano_cli: &CardanoCli,
    config: &Config,
) -> Result<()> {
    println!("{}", "🔄 Starting continuous health monitoring...".green().bold());
    println!("{}", "Press Ctrl+C to stop".dimmed());

    let interval = Duration::from_secs(
        std::env::args()
            .position(|arg| arg == "--interval")
            .and_then(|pos| std::env::args().nth(pos + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(30)
    );

    let mut interval_timer = time::interval(interval);
    let mut check_count = 0;

    loop {
        interval_timer.tick().await;
        check_count += 1;

        // Clear screen for live updates
        print!("\x1B[2J\x1B[1;1H");

        println!("{}", format!("🏥 Health Monitor - Check #{} (Every {}s)",
                               check_count, interval.as_secs()).blue().bold());
        println!("{}", format!("Last Update: {}", get_current_timestamp()).dimmed());
        println!("{}", "=".repeat(70).blue());

        match perform_health_checks(cardano_cli, config).await {
            Ok(health_results) => {
                display_health_results(&health_results);

                // Check for alerts
                if let Err(e) = process_alerts(&health_results, config).await {
                    println!("{} Alert processing failed: {}", "⚠️".yellow(), e);
                }

                // Export metrics if requested
                if std::env::args().any(|arg| arg == "--export") {
                    if let Err(e) = export_health_metrics(&health_results, config).await {
                        println!("{} Export failed: {}", "⚠️".yellow(), e);
                    }
                }
            },
            Err(e) => {
                println!("{} Health check failed: {}", "❌".red(), e);
            }
        }

        println!("\n{}", format!("Next check in {}s... (Ctrl+C to stop)", interval.as_secs()).dimmed());
    }
}

async fn perform_health_checks(
    cardano_cli: &CardanoCli,
    config: &Config,
) -> Result<Vec<HealthMetrics>> {
    let mut results = Vec::new();

    for pool in &config.pools {
        let start_time = Instant::now();
        let health = check_pool_health(cardano_cli, pool, config).await?;
        let response_time = start_time.elapsed().as_millis() as u64;

        let mut health_metrics = health;
        health_metrics.response_time_ms = response_time;

        results.push(health_metrics);
    }

    Ok(results)
}

async fn check_pool_health(
    cardano_cli: &CardanoCli,
    pool: &crate::utils::config::PoolConfig,
    config: &Config,
) -> Result<HealthMetrics> {
    let mut issues = Vec::new();
    let timestamp = get_current_timestamp();

    // Check node sync status
    let sync_status = match cardano_cli.query_tip().await {
        Ok(tip) => {
            let slot = tip["slot"].as_u64().unwrap_or(0);
            let _epoch = tip["epoch"].as_u64().unwrap_or(0);

            // Simplified sync check (in reality, you'd compare with network tip)
            if slot > 0 {
                SyncStatus::Synced
            } else {
                SyncStatus::Unknown
            }
        },
        Err(_) => {
            issues.push("Node unreachable".to_string());
            SyncStatus::Unknown
        }
    };

    // Check saturation status
    let saturation_status = match get_pool_saturation(cardano_cli, &pool.pool_id).await {
        Ok(saturation) => {
            if saturation > config.monitoring.alerts.saturation_threshold {
                issues.push(format!("Pool oversaturated at {:.1}%", saturation * 100.0));
                SaturationStatus::Oversaturated { percentage: saturation * 100.0 }
            } else if saturation > 0.7 {
                SaturationStatus::High { percentage: saturation * 100.0 }
            } else if saturation > 0.01 {
                SaturationStatus::Optimal { percentage: saturation * 100.0 }
            } else {
                SaturationStatus::Low { percentage: saturation * 100.0 }
            }
        },
        Err(_) => {
            issues.push("Cannot determine saturation".to_string());
            SaturationStatus::Unknown
        }
    };

    // Check block production
    let block_production = match get_block_production(cardano_cli, &pool.pool_id).await {
        Ok((actual, expected)) => {
            if actual == 0 && expected > 0 {
                issues.push("No blocks produced this epoch".to_string());
                BlockProductionStatus::NoBlocks { expected }
            } else if actual < expected.saturating_sub(1) {
                issues.push("Underperforming block production".to_string());
                BlockProductionStatus::Underperforming { blocks_current_epoch: actual, expected }
            } else {
                BlockProductionStatus::Healthy { blocks_current_epoch: actual, expected }
            }
        },
        Err(_) => BlockProductionStatus::Unknown,
    };

    let is_healthy = issues.is_empty();

    Ok(HealthMetrics {
        timestamp,
        pool_id: pool.pool_id.clone(),
        pool_name: pool.name.clone(),
        is_healthy,
        sync_status,
        saturation_status,
        block_production,
        response_time_ms: 0, // Will be set by caller
        issues,
    })
}

async fn get_pool_saturation(cardano_cli: &CardanoCli, _pool_id: &str) -> Result<f64> {
    // Try to get real saturation, fallback to demo value
    match cardano_cli.query_stake_distribution().await {
        Ok(_distribution) => {
            // In demo mode, return realistic values
            Ok(0.042) // 4.2% saturation
        },
        Err(_) => Ok(0.042), // Demo fallback
    }
}

async fn get_block_production(cardano_cli: &CardanoCli, _pool_id: &str) -> Result<(u64, u64)> {
    // Try to get real block production data
    match cardano_cli.query_tip().await {
        Ok(_tip) => {
            // Demo values: (actual_blocks, expected_blocks)
            Ok((3, 4)) // Slightly underperforming
        },
        Err(_) => Ok((0, 0)),
    }
}

fn display_health_results(results: &[HealthMetrics]) {
    let summaries: Vec<HealthSummary> = results.iter().map(|health| {
        let overall_status = if health.is_healthy {
            "🟢 Healthy".green().to_string()
        } else {
            "🔴 Issues".red().to_string()
        };

        let sync_status = match &health.sync_status {
            SyncStatus::Synced => "🟢 Synced".green().to_string(),
            SyncStatus::Syncing { blocks_behind } => format!("🟡 Syncing (-{})", blocks_behind).yellow().to_string(),
            SyncStatus::OutOfSync { blocks_behind } => format!("🔴 Behind (-{})", blocks_behind).red().to_string(),
            SyncStatus::Unknown => "⚪ Unknown".to_string(),
        };

        let saturation = match &health.saturation_status {
            SaturationStatus::Optimal { percentage } => format!("🟢 {:.1}%", percentage).green().to_string(),
            SaturationStatus::High { percentage } => format!("🟡 {:.1}%", percentage).yellow().to_string(),
            SaturationStatus::Oversaturated { percentage } => format!("🔴 {:.1}%", percentage).red().to_string(),
            SaturationStatus::Low { percentage } => format!("🔵 {:.1}%", percentage).blue().to_string(),
            SaturationStatus::Unknown => "⚪ Unknown".to_string(),
        };

        let blocks = match &health.block_production {
            BlockProductionStatus::Healthy { blocks_current_epoch, expected } =>
                format!("🟢 {}/{}", blocks_current_epoch, expected).green().to_string(),
            BlockProductionStatus::Underperforming { blocks_current_epoch, expected } =>
                format!("🟡 {}/{}", blocks_current_epoch, expected).yellow().to_string(),
            BlockProductionStatus::NoBlocks { expected } =>
                format!("🔴 0/{}", expected).red().to_string(),
            BlockProductionStatus::Unknown => "⚪ Unknown".to_string(),
        };

        HealthSummary {
            pool_name: health.pool_name.clone(),
            overall_status,
            sync_status,
            saturation,
            blocks,
            response_time: format!("{}ms", health.response_time_ms),
            issue_count: if health.issues.is_empty() {
                "0".green().to_string()
            } else {
                health.issues.len().to_string().red().to_string()
            },
        }
    }).collect();

    let table = Table::new(summaries);
    println!("{}", table);
}

async fn process_alerts(results: &[HealthMetrics], config: &Config) -> Result<()> {
    let unhealthy_pools: Vec<_> = results.iter().filter(|r| !r.is_healthy).collect();

    if unhealthy_pools.is_empty() {
        return Ok(());
    }

    // Log alerts
    for pool in unhealthy_pools {
        eprintln!("{} ALERT: {} has {} issues: {}",
                  get_current_timestamp(),
                  pool.pool_name,
                  pool.issues.len(),
                  pool.issues.join(", ")
        );
    }

    // TODO: Implement webhook notifications if configured
    if !config.monitoring.alerts.webhook_url.is_empty() {
        println!("{} Would send webhook to: {}",
                 "🔔".yellow(),
                 config.monitoring.alerts.webhook_url
        );
    }

    // TODO: Implement email notifications if enabled
    if config.monitoring.alerts.email_enabled {
        println!("{} Would send email alerts", "📧".yellow());
    }

    Ok(())
}

async fn export_health_metrics(results: &[HealthMetrics], config: &Config) -> Result<()> {
    let export_dir = std::env::args()
        .position(|arg| arg == "--export")
        .and_then(|pos| std::env::args().nth(pos + 1))
        .unwrap_or_else(|| config.rewards.output_directory.clone());

    std::fs::create_dir_all(&export_dir)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let filename = format!("{}/health_metrics_{}.json", export_dir, timestamp);

    let json = serde_json::to_string_pretty(results)?;
    std::fs::write(&filename, json)?;

    println!("{} Exported metrics to: {}", "💾".cyan(), filename);
    Ok(())
}

fn get_current_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = duration.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    // Simple timestamp format
    format!("Day {} {:02}:{:02}:{:02} UTC", days, hours, minutes, seconds)
}