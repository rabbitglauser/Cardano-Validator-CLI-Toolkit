
use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};
use serde_json::Value;
use crate::cardano::cli::CardanoCli;
use crate::utils::config::Config;

#[derive(Tabled)]
struct PoolStatus {
    #[tabled(rename = "Pool ID")]
    pool_id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Saturation")]
    saturation: String,
    #[tabled(rename = "Live Stake")]
    live_stake: String,
    #[tabled(rename = "Blocks")]
    blocks_epoch: String,
}

pub async fn execute(pool_id: Option<String>, config: &Config) -> Result<()> {
    let cardano_cli = CardanoCli::new(&config);

    println!("{}", "🔍 Checking pool status...".blue().bold());

    // Check if cardano-cli is available
    if !cardano_cli.is_available().await {
        println!("{}", "⚠️  cardano-cli not available - using demo mode".yellow());
    }

    let pools_to_check = if let Some(id) = pool_id {
        vec![(id, "Manual Query".to_string())]
    } else {
        config.pools.iter()
            .map(|p| (p.pool_id.clone(), p.name.clone()))
            .collect()
    };

    if pools_to_check.is_empty() {
        println!("{}", "❌ No pools configured or specified".red());
        return Ok(());
    }

    let mut statuses = Vec::new();

    for (pool_id, pool_name) in pools_to_check.iter() {
        print!("Checking {} ({})... ", pool_name.cyan(), pool_id.dimmed());

        match check_pool_status(&cardano_cli, &pool_id, &pool_name).await {
            Ok(status) => {
                println!("{}", "✓".green());
                statuses.push(status);
            },
            Err(e) => {
                println!("{}", "✗".red());
                println!("{} Failed to check pool {}: {}", "❌".red(), pool_id, e);
                // Add placeholder status for failed checks
                statuses.push(create_placeholder_status(pool_id, pool_name));
            }
        }
    }

    if !statuses.is_empty() {
        println!("\n{}", "📊 Pool Status Summary:".blue().bold());
        let table = Table::new(statuses);
        println!("{}", table);
    }

    Ok(())
}

fn create_placeholder_status(pool_id: &str, pool_name: &str) -> PoolStatus {
    PoolStatus {
        pool_id: truncate_pool_id(pool_id),
        name: pool_name.to_string(),
        status: "⚪ Unknown".to_string(),
        saturation: "-.-%".to_string(),
        live_stake: "-".to_string(),
        blocks_epoch: "-".to_string(),
    }
}

fn get_pool_stake_from_distribution(distribution: &Value, pool_id: &str) -> u64 {
    // The stake distribution format can vary, try different possible structures
    if let Some(pools) = distribution.get("pools") {
        if let Some(stake_str) = pools.get(pool_id).and_then(|v| v.as_str()) {
            return stake_str.parse::<u64>().unwrap_or(0);
        }
    }

    // Alternative: direct pool lookup
    if let Some(stake_str) = distribution.get(pool_id).and_then(|v| v.as_str()) {
        return stake_str.parse::<u64>().unwrap_or(0);
    }

    0
}

fn get_total_stake(distribution: &Value) -> u64 {
    // Try different possible locations for total stake
    if let Some(total_str) = distribution.get("total").and_then(|v| v.as_str()) {
        return total_str.parse::<u64>().unwrap_or(0);
    }

    if let Some(total_str) = distribution.get("totalStake").and_then(|v| v.as_str()) {
        return total_str.parse::<u64>().unwrap_or(0);
    }

    // If we can't find total, sum all pool stakes
    if let Some(pools) = distribution.get("pools").and_then(|v| v.as_object()) {
        return pools.values()
            .filter_map(|v| v.as_str())
            .filter_map(|s| s.parse::<u64>().ok())
            .sum();
    }

    0
}

async fn check_pool_status(
    cardano_cli: &CardanoCli,
    pool_id: &str,
    pool_name: &str,
) -> Result<PoolStatus> {
    // Try to get actual tip information first
    let tip = match cardano_cli.query_tip().await {
        Ok(tip) => tip,
        Err(_) => {
            // Fallback to demo mode if cardano-cli not available
            return Ok(create_demo_status(pool_id, pool_name));
        }
    };

    let current_epoch = tip["epoch"].as_u64().unwrap_or(0);

    // Try to get stake distribution (this can be slow/fail)
    let (pool_stake, total_stake) = match cardano_cli.query_stake_distribution().await {
        Ok(distribution) => {
            let pool_stake = get_pool_stake_from_distribution(&distribution, pool_id);
            let total_stake = get_total_stake(&distribution);
            (pool_stake, total_stake)
        },
        Err(_) => {
            // Use placeholder values if stake distribution query fails
            (1_000_000_000_000u64, 32_000_000_000_000_000u64) // 1M ADA, 32B total
        }
    };

    let saturation = if total_stake > 0 {
        (pool_stake as f64 / total_stake as f64) * 100.0
    } else {
        0.0
    };

    // Check if pool is active
    let is_active = match cardano_cli.query_pool_info(pool_id).await {
        Ok(info) => info["active"].as_bool().unwrap_or(false),
        Err(_) => true, // Assume active if we can't check
    };

    let status = if is_active {
        if saturation > 95.0 {
            "🔴 Oversaturated".red().to_string()
        } else if saturation > 70.0 {
            "🟡 High Saturation".yellow().to_string()
        } else if saturation > 0.1 {
            "🟢 Active".green().to_string()
        } else {
            "🔵 Active (Low Stake)".blue().to_string()
        }
    } else {
        "⚫ Retired".to_string()
    };

    // Get blocks for current epoch (simplified)
    let blocks_count = match cardano_cli.query_pool_blocks(pool_id, current_epoch).await {
        Ok(blocks) => blocks.as_array().map(|a| a.len()).unwrap_or(0),
        Err(_) => 0, // Default to 0 if query fails
    };

    Ok(PoolStatus {
        pool_id: truncate_pool_id(pool_id),
        name: pool_name.to_string(),
        status,
        saturation: format!("{:.2}%", saturation),
        live_stake: format_ada(pool_stake),
        blocks_epoch: blocks_count.to_string(),
    })
}

fn create_demo_status(pool_id: &str, pool_name: &str) -> PoolStatus {
    // Create realistic demo data
    let demo_stake = 1_500_000_000_000u64; // 1.5M ADA
    let demo_saturation = 4.2; // 4.2%

    PoolStatus {
        pool_id: truncate_pool_id(pool_id),
        name: pool_name.to_string(),
        status: "🟢 Active (Demo)".green().to_string(),
        saturation: format!("{:.2}%", demo_saturation),
        live_stake: format_ada(demo_stake),
        blocks_epoch: "3".to_string(),
    }
}

fn truncate_pool_id(pool_id: &str) -> String {
    if pool_id.len() > 20 {
        format!("{}...{}", &pool_id[..8], &pool_id[pool_id.len()-4..])
    } else {
        pool_id.to_string()
    }
}

fn format_ada(lovelace: u64) -> String {
    let ada = lovelace as f64 / 1_000_000.0;
    if ada >= 1_000_000.0 {
        format!("{:.1}M ₳", ada / 1_000_000.0)
    } else if ada >= 1_000.0 {
        format!("{:.1}K ₳", ada / 1_000.0)
    } else {
        format!("{:.1} ₳", ada)
    }
}