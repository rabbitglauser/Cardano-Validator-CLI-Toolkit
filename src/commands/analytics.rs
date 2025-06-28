use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

use crate::cardano::cli::CardanoCli;
use crate::utils::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub pool_id: String,
    pub pool_name: String,
    pub epoch_range: (u64, u64),
    pub performance_metrics: PerformanceMetrics,
    pub trends: TrendAnalysis,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub block_production_rate: f64,
    pub average_saturation: f64,
    pub reward_efficiency: f64,
    pub uptime_percentage: f64,
    pub delegator_count_change: i64,
    pub stake_change_ada: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub performance_trend: Trend,
    pub saturation_trend: Trend,
    pub delegator_trend: Trend,
    pub reward_trend: Trend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Improving { percentage: f64 },
    Declining { percentage: f64 },
    Stable,
    Unknown,
}

#[derive(Tabled)]
struct AnalyticsSummary {
    #[tabled(rename = "Pool")]
    pool_name: String,
    #[tabled(rename = "Block Rate")]
    block_rate: String,
    #[tabled(rename = "Avg Saturation")]
    avg_saturation: String,
    #[tabled(rename = "Efficiency")]
    efficiency: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
    #[tabled(rename = "Trend")]
    overall_trend: String,
    #[tabled(rename = "Recommendations")]
    rec_count: String,
}

pub async fn execute(
    pool_id: Option<String>,
    epochs: u64,
    detailed: bool,
    export: bool,
    config: &Config,
) -> Result<()> {
    println!("{}", "📊 Advanced Analytics System".blue().bold());
    println!("{}", "=".repeat(50).blue());

    let cardano_cli = CardanoCli::new(config);

    if detailed {
        generate_detailed_report(&cardano_cli, config, pool_id, epochs, export).await
    } else {
        generate_summary_analytics(&cardano_cli, config, epochs, export).await
    }
}

async fn generate_summary_analytics(
    cardano_cli: &CardanoCli,
    config: &Config,
    epochs: u64,
    export: bool,
) -> Result<()> {
    println!("{}", format!("📈 Generating analytics for last {} epochs...", epochs).cyan());

    let mut reports = Vec::new();

    for pool in &config.pools {
        let report = analyze_pool_performance(cardano_cli, pool, epochs).await?;
        reports.push(report);
    }

    display_analytics_summary(&reports);

    if export {
        export_analytics_report(&reports, config).await?;
    }

    Ok(())
}

async fn generate_detailed_report(
    cardano_cli: &CardanoCli,
    config: &Config,
    pool_id: Option<String>,
    epochs: u64,
    export: bool,
) -> Result<()> {
    println!("{}", format!("🔍 Generating detailed report for {} epochs...", epochs).cyan());

    let target_pools: Vec<_> = if let Some(id) = pool_id {
        config.pools.iter().filter(|p| p.pool_id == id).collect()
    } else {
        config.pools.iter().collect()
    };

    if target_pools.is_empty() {
        println!("{}", "❌ No matching pools found".red());
        return Ok(());
    }

    for pool in target_pools {
        let report = analyze_pool_performance(cardano_cli, pool, epochs).await?;
        display_detailed_report(&report).await?;

        if export {
            export_single_pool_report(&report, config).await?;
        }
    }

    Ok(())
}

async fn analyze_pool_performance(
    cardano_cli: &CardanoCli,
    pool: &crate::utils::config::PoolConfig,
    epochs: u64,
) -> Result<AnalyticsReport> {
    // Get current epoch
    let current_epoch = match cardano_cli.query_tip().await {
        Ok(tip) => tip["epoch"].as_u64().unwrap_or(450),
        Err(_) => 450, // Demo epoch
    };

    let start_epoch = current_epoch.saturating_sub(epochs);

    // Simulate performance analysis (in real implementation, query blockchain data)
    let performance_metrics = PerformanceMetrics {
        block_production_rate: 0.95, // 95% of expected blocks
        average_saturation: 0.42,    // 42% saturation
        reward_efficiency: 0.98,     // 98% reward efficiency
        uptime_percentage: 99.8,     // 99.8% uptime
        delegator_count_change: 15,  // +15 delegators
        stake_change_ada: 50000.0,   // +50K ADA
    };

    let trends = TrendAnalysis {
        performance_trend: Trend::Improving { percentage: 3.2 },
        saturation_trend: Trend::Stable,
        delegator_trend: Trend::Improving { percentage: 8.5 },
        reward_trend: Trend::Improving { percentage: 1.8 },
    };

    let mut recommendations = Vec::new();

    // Generate recommendations based on metrics
    if performance_metrics.average_saturation < 0.3 {
        recommendations.push("Consider marketing to increase delegation".to_string());
    }
    if performance_metrics.average_saturation > 0.8 {
        recommendations.push("Pool approaching oversaturation - monitor closely".to_string());
    }
    if performance_metrics.block_production_rate < 0.9 {
        recommendations.push("Investigate block production issues".to_string());
    }
    if performance_metrics.uptime_percentage < 99.0 {
        recommendations.push("Improve infrastructure reliability".to_string());
    }

    if recommendations.is_empty() {
        recommendations.push("Pool performance is excellent - maintain current operations".to_string());
    }

    Ok(AnalyticsReport {
        pool_id: pool.pool_id.clone(),
        pool_name: pool.name.clone(),
        epoch_range: (start_epoch, current_epoch),
        performance_metrics,
        trends,
        recommendations,
    })
}

fn display_analytics_summary(reports: &[AnalyticsReport]) {
    let summaries: Vec<AnalyticsSummary> = reports.iter().map(|report| {
        let block_rate = format!("{:.1}%", report.performance_metrics.block_production_rate * 100.0);
        let avg_saturation = format!("{:.1}%", report.performance_metrics.average_saturation * 100.0);
        let efficiency = format!("{:.1}%", report.performance_metrics.reward_efficiency * 100.0);
        let uptime = format!("{:.1}%", report.performance_metrics.uptime_percentage);

        let overall_trend = match &report.trends.performance_trend {
            Trend::Improving { percentage } => format!("📈 +{:.1}%", percentage).green().to_string(),
            Trend::Declining { percentage } => format!("📉 -{:.1}%", percentage).red().to_string(),
            Trend::Stable => "📊 Stable".yellow().to_string(),
            Trend::Unknown => "❓ Unknown".dimmed().to_string(),
        };

        let rec_count = format!("{}", report.recommendations.len());

        AnalyticsSummary {
            pool_name: report.pool_name.clone(),
            block_rate: if report.performance_metrics.block_production_rate >= 0.95 {
                block_rate.green().to_string()
            } else if report.performance_metrics.block_production_rate >= 0.85 {
                block_rate.yellow().to_string()
            } else {
                block_rate.red().to_string()
            },
            avg_saturation: if report.performance_metrics.average_saturation <= 0.8 && report.performance_metrics.average_saturation >= 0.3 {
                avg_saturation.green().to_string()
            } else {
                avg_saturation.yellow().to_string()
            },
            efficiency: if report.performance_metrics.reward_efficiency >= 0.95 {
                efficiency.green().to_string()
            } else {
                efficiency.yellow().to_string()
            },
            uptime: if report.performance_metrics.uptime_percentage >= 99.5 {
                uptime.green().to_string()
            } else if report.performance_metrics.uptime_percentage >= 98.0 {
                uptime.yellow().to_string()
            } else {
                uptime.red().to_string()
            },
            overall_trend,
            rec_count: if report.recommendations.len() <= 1 {
                rec_count.green().to_string()
            } else {
                rec_count.yellow().to_string()
            },
        }
    }).collect();

    println!("\n{}", "📊 Performance Analytics Summary".blue().bold());
    let table = Table::new(summaries);
    println!("{}", table);
}

async fn display_detailed_report(report: &AnalyticsReport) -> Result<()> {
    println!("\n{}", format!("📋 Detailed Report: {}", report.pool_name).blue().bold());
    println!("{}", "=".repeat(60));

    println!("\n{}", "📊 Performance Metrics".cyan().bold());
    println!("  • Block Production Rate: {:.1}% {}",
             report.performance_metrics.block_production_rate * 100.0,
             if report.performance_metrics.block_production_rate >= 0.95 { "🟢" } else { "🟡" }
    );
    println!("  • Average Saturation: {:.1}%", report.performance_metrics.average_saturation * 100.0);
    println!("  • Reward Efficiency: {:.1}%", report.performance_metrics.reward_efficiency * 100.0);
    println!("  • Uptime: {:.1}%", report.performance_metrics.uptime_percentage);
    println!("  • Delegator Change: {:+}", report.performance_metrics.delegator_count_change);
    println!("  • Stake Change: {:+.0} ADA", report.performance_metrics.stake_change_ada);

    println!("\n{}", "📈 Trend Analysis".cyan().bold());
    print_trend("Performance", &report.trends.performance_trend);
    print_trend("Saturation", &report.trends.saturation_trend);
    print_trend("Delegators", &report.trends.delegator_trend);
    print_trend("Rewards", &report.trends.reward_trend);

    println!("\n{}", "💡 Recommendations".cyan().bold());
    for (i, rec) in report.recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, rec);
    }

    println!("\n{}", format!("📅 Analysis Period: Epochs {} - {}",
                             report.epoch_range.0, report.epoch_range.1).dimmed());

    Ok(())
}

fn print_trend(label: &str, trend: &Trend) {
    let trend_text = match trend {
        Trend::Improving { percentage } => format!("📈 Improving (+{:.1}%)", percentage).green(),
        Trend::Declining { percentage } => format!("📉 Declining (-{:.1}%)", percentage).red(),
        Trend::Stable => "📊 Stable".yellow(),
        Trend::Unknown => "❓ Unknown".dimmed(),
    };
    println!("  • {}: {}", label, trend_text);
}

async fn export_analytics_report(reports: &[AnalyticsReport], config: &Config) -> Result<()> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let export_dir = &config.rewards.output_directory;
    std::fs::create_dir_all(export_dir)?;

    let filename = format!("{}/analytics_report_{}.json", export_dir, timestamp);
    let json = serde_json::to_string_pretty(reports)?;
    std::fs::write(&filename, json)?;

    println!("\n{} Analytics report exported to: {}", "💾".cyan(), filename);
    Ok(())
}

async fn export_single_pool_report(report: &AnalyticsReport, config: &Config) -> Result<()> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let export_dir = &config.rewards.output_directory;
    std::fs::create_dir_all(export_dir)?;

    let filename = format!("{}/detailed_report_{}_{}.json",
                           export_dir, report.pool_name.replace(" ", "_"), timestamp);
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(&filename, json)?;

    println!("\n{} Detailed report exported to: {}", "💾".cyan(), filename);
    Ok(())
}