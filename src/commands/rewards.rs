use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use crate::cardano::cli::CardanoCli;
use crate::utils::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsReport {
    pub pool_id: String,
    pub pool_name: String,
    pub epoch: u64,
    pub total_rewards: f64,
    pub pool_rewards: f64,
    pub delegator_rewards: f64,
    pub fees: f64,
    pub delegator_count: u64,
    pub average_reward_per_delegator: f64,
}

#[derive(Tabled)]
struct RewardsSummary {
    #[tabled(rename = "Pool")]
    pool_name: String,
    #[tabled(rename = "Epoch")]
    epoch: String,
    #[tabled(rename = "Total Rewards")]
    total_rewards: String,
    #[tabled(rename = "Pool Share")]
    pool_share: String,
    #[tabled(rename = "Delegator Share")]
    delegator_share: String,
    #[tabled(rename = "Avg per Delegator")]
    avg_per_delegator: String,
}

pub async fn execute(epoch: Option<u64>, detailed: bool, config: &Config) -> Result<()> {
    println!("{}", "💰 Rewards Calculation System".blue().bold());
    println!("{}", "=".repeat(50).blue());

    let cardano_cli = CardanoCli::new(config);

    let target_epoch = epoch.unwrap_or_else(|| {
        // Get current epoch - 1 (rewards are calculated for previous epoch)
        450 // Demo epoch
    });

    if detailed {
        generate_detailed_rewards_report(&cardano_cli, config, target_epoch).await
    } else {
        generate_rewards_summary(&cardano_cli, config, target_epoch).await
    }
}

async fn generate_rewards_summary(
    cardano_cli: &CardanoCli,
    config: &Config,
    epoch: u64,
) -> Result<()> {
    println!("{}", format!("📊 Calculating rewards for epoch {}...", epoch).cyan());

    let mut reports = Vec::new();

    for pool in &config.pools {
        let report = calculate_pool_rewards(cardano_cli, pool, epoch, config).await?;
        reports.push(report);
    }

    display_rewards_summary(&reports);

    // Auto-export in JSON format
    export_rewards_report(&reports, config).await?;

    Ok(())
}

async fn generate_detailed_rewards_report(
    cardano_cli: &CardanoCli,
    config: &Config,
    epoch: u64,
) -> Result<()> {
    println!("{}", format!("🔍 Generating detailed rewards report for epoch {}...", epoch).cyan());

    for pool in &config.pools {
        let report = calculate_pool_rewards(cardano_cli, pool, epoch, config).await?;
        display_detailed_rewards_report(&report).await?;
    }

    Ok(())
}

async fn calculate_pool_rewards(
    _cardano_cli: &CardanoCli,
    pool: &crate::utils::config::PoolConfig,
    epoch: u64,
    config: &Config,
) -> Result<RewardsReport> {
    // Simulate rewards calculation (in real implementation, query blockchain data)
    let total_rewards = 1500.0; // 1500 ADA total rewards
    let pool_fee_percentage = 5.0; // 5% pool fee
    let pool_rewards = total_rewards * (pool_fee_percentage / 100.0);
    let delegator_rewards = total_rewards - pool_rewards;
    let fees = if config.rewards.include_fees { 2.17 } else { 0.0 }; // Transaction fees
    let delegator_count = 250; // 250 delegators
    let average_reward_per_delegator = delegator_rewards / delegator_count as f64;

    Ok(RewardsReport {
        pool_id: pool.pool_id.clone(),
        pool_name: pool.name.clone(),
        epoch,
        total_rewards,
        pool_rewards,
        delegator_rewards,
        fees,
        delegator_count,
        average_reward_per_delegator,
    })
}

fn display_rewards_summary(reports: &[RewardsReport]) {
    let summaries: Vec<RewardsSummary> = reports.iter().map(|report| {
        RewardsSummary {
            pool_name: report.pool_name.clone(),
            epoch: report.epoch.to_string(),
            total_rewards: format!("{:.2} ADA", report.total_rewards),
            pool_share: format!("{:.2} ADA", report.pool_rewards),
            delegator_share: format!("{:.2} ADA", report.delegator_rewards),
            avg_per_delegator: format!("{:.4} ADA", report.average_reward_per_delegator),
        }
    }).collect();

    println!("\n{}", "💰 Rewards Summary".blue().bold());
    let table = Table::new(summaries);
    println!("{}", table);

    // Display totals
    let total_rewards: f64 = reports.iter().map(|r| r.total_rewards).sum();
    let total_pool_rewards: f64 = reports.iter().map(|r| r.pool_rewards).sum();
    let total_delegator_rewards: f64 = reports.iter().map(|r| r.delegator_rewards).sum();

    println!("\n{}", "📊 Overall Totals".cyan().bold());
    println!("  • Total Rewards: {:.2} ADA", total_rewards);
    println!("  • Total Pool Fees: {:.2} ADA", total_pool_rewards);
    println!("  • Total Delegator Rewards: {:.2} ADA", total_delegator_rewards);
}

async fn display_detailed_rewards_report(report: &RewardsReport) -> Result<()> {
    println!("\n{}", format!("💎 Detailed Rewards: {} (Epoch {})",
                             report.pool_name, report.epoch).blue().bold());
    println!("{}", "=".repeat(60));

    println!("\n{}", "💰 Reward Breakdown".cyan().bold());
    println!("  • Total Epoch Rewards: {:.2} ADA", report.total_rewards);
    println!("  • Pool Operator Share: {:.2} ADA ({:.1}%)",
             report.pool_rewards,
             (report.pool_rewards / report.total_rewards) * 100.0
    );
    println!("  • Delegator Share: {:.2} ADA ({:.1}%)",
             report.delegator_rewards,
             (report.delegator_rewards / report.total_rewards) * 100.0
    );
    println!("  • Transaction Fees: {:.2} ADA", report.fees);

    println!("\n{}", "👥 Delegator Statistics".cyan().bold());
    println!("  • Total Delegators: {}", report.delegator_count);
    println!("  • Average Reward per Delegator: {:.4} ADA", report.average_reward_per_delegator);
    println!("  • Estimated Annual Return: ~{:.1}%", report.average_reward_per_delegator * 73.0 / 1000.0 * 100.0); // Rough estimate

    println!("\n{}", "📈 Performance Metrics".cyan().bold());
    println!("  • ROA (Return on ADA): {:.2}%", (report.total_rewards / 50000.0) * 100.0); // Assuming 50K ADA stake
    println!("  • Effective Pool Margin: {:.1}%", (report.pool_rewards / report.total_rewards) * 100.0);

    Ok(())
}

async fn export_rewards_report(reports: &[RewardsReport], config: &Config) -> Result<()> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let export_dir = &config.rewards.output_directory;
    std::fs::create_dir_all(export_dir)?;

    let filename = format!("{}/rewards_report_{}.json", export_dir, timestamp);
    let json = serde_json::to_string_pretty(reports)?;
    std::fs::write(&filename, json)?;

    println!("\n{} Rewards report exported to: {}", "💾".cyan(), filename);
    Ok(())
}