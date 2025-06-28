use anyhow::Result;
use colored::*;
use tabled::{Table, Tabled};
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
    let cardano_cli = CardanoCli::new(config);

    println!("{}", "🔍 Checking pool status...".blue().bold());

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

    for (pool_id, pool_name) in pools_to_check {
        match check_pool_status(&cardano_cli, &pool_id, &pool_name).await {
            Ok(status) => statuses.push(status),
            Err(e) => {
                println!("{} Failed to check pool {}: {}", "❌".red(), pool_id, e);
            }
        }
    }

    if !statuses.is_empty() {
        let table = Table::new(statuses);
        println!("\n{}", table);
    }

    Ok(())
}

async fn check_pool_status(
    cardano_cli: &CardanoCli,
    pool_id: &str,
    pool_name: &str,
) -> Result<PoolStatus> {
    // Query current tip to get current epoch
    let tip = cardano_cli.query_tip().await?;
    let current_epoch = tip["epoch"].as_u64().unwrap_or(0);

    // Query stake distribution to get pool info
    let stake_dist = cardano_cli.query_stake_distribution().await?;

    let pool_stake = stake_dist[pool_id]["stake"].as_str()
        .unwrap_or("0")
        .parse::<u64>()
        .unwrap_or(0);

    let total_stake = stake_dist.as_object().unwrap()
        .values()
        .filter_map(|v| v["stake"].as_str())
        .filter_map(|s| s.parse::<u64>().ok())
        .sum::<u64>();

    let saturation = if total_stake > 0 {
        (pool_stake as f64 / total_stake as f64) * 100.0
    } else {
        0.0
    };

    // Query pool parameters
    let pool_params = cardano_cli.query_pool_params(pool_id).await?;

    let is_active = !pool_params.is_null();
    let status = if is_active {
        if saturation > 95.0 {
            "🔴 Oversaturated".red().to_string()
        } else if saturation > 70.0 {
            "🟡 High Saturation".yellow().to_string()
        } else {
            "🟢 Active".green().to_string()
        }
    } else {
        "⚫ Retired".to_string()
    };

    // Format stake amount
    let live_stake_formatted = format_ada(pool_stake);

    // For blocks, we'd need to query leadership schedule or use a different approach
    let blocks_epoch = "N/A".to_string(); // Placeholder for now

    Ok(PoolStatus {
        pool_id: pool_id.to_string(),
        name: pool_name.to_string(),
        status,
        saturation: format!("{:.2}%", saturation),
        live_stake: live_stake_formatted,
        blocks_epoch,
    })
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