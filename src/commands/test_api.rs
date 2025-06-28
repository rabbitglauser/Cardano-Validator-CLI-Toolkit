use anyhow::Result;
use colored::*;
use crate::utils::config::Config;

pub async fn execute(config: &Config) -> Result<()> {
    println!("{}", "🧪 Testing Blockfrost API Connection".blue().bold());
    println!("{}", "=".repeat(50).blue());

    // Import BlockfrostClient directly
    use crate::cardano::blockfrost::BlockfrostClient;

    match BlockfrostClient::new(config) {
        Some(client) => {
            println!("{} Blockfrost client created successfully", "✅".green());

            // Test 1: Get network info
            println!("\n{}", "📡 Testing network info...".cyan());
            match client.get_network_info().await {
                Ok(network_info) => {
                    println!("{} Network info retrieved!", "✅".green());
                    println!("   Supply: {}", network_info["supply"]["circulating"].as_str().unwrap_or("N/A"));
                    println!("   Max Supply: {}", network_info["supply"]["max"].as_str().unwrap_or("N/A"));
                    println!("   Stake: {}", network_info["stake"]["active"].as_str().unwrap_or("N/A"));
                },
                Err(e) => {
                    println!("{} Failed to get network info: {}", "❌".red(), e);
                    return Err(e);
                }
            }

            // Test 2: Get latest epoch
            println!("\n{}", "📅 Testing latest epoch...".cyan());
            match client.get_latest_epoch().await {
                Ok(epoch_info) => {
                    println!("{} Latest epoch retrieved!", "✅".green());
                    println!("   Epoch: {}", epoch_info["epoch"].as_u64().unwrap_or(0));
                    println!("   Start Time: {}", epoch_info["start_time"].as_u64().unwrap_or(0));
                    println!("   End Time: {}", epoch_info["end_time"].as_u64().unwrap_or(0));
                },
                Err(e) => {
                    println!("{} Failed to get epoch info: {}", "❌".red(), e);
                    return Err(e);
                }
            }

            // Test 3: Get first 5 pools
            println!("\n{}", "🏊 Testing pool data...".cyan());
            match client.get_all_pools().await {
                Ok(pools) => {
                    println!("{} Pool list retrieved!", "✅".green());
                    if let Some(pools_array) = pools.as_array() {
                        println!("   Found {} pools", pools_array.len());
                        for (i, pool_id) in pools_array.iter().take(3).enumerate() {
                            if let Some(pool_id_str) = pool_id.as_str() {
                                println!("   {}. {}", i + 1, pool_id_str);
                            }
                        }
                    }
                },
                Err(e) => {
                    println!("{} Failed to get pool list: {}", "❌".red(), e);
                    return Err(e);
                }
            }

            println!("\n{} All API tests passed! Your connection is working perfectly! 🚀", "🎉".green().bold());
        },
        None => {
            println!("{} No Blockfrost configuration found in config.toml", "❌".red());
            println!("Make sure you have the [blockfrost] section in your config.toml");
        }
    }

    Ok(())
}