use anyhow::Result;
use std::io::{self, Write};
use crate::utils::config::{Config, PoolConfig, BlockfrostConfig, CardanoConfig, MonitoringConfig, RewardsConfig, AlertsConfig};
use crate::cardano::blockfrost::BlockfrostClient;
use std::path::PathBuf;

pub async fn execute() -> Result<()> {
    println!("🚀 Welcome to Cardano Validator CLI Setup!");
    println!("This wizard will help you configure your stake pool monitoring.\n");

    // Get Blockfrost configuration
    let blockfrost_config = setup_blockfrost().await?;

    // Get pool configuration
    let pools = setup_pools().await?;

    // Test the configuration
    println!("\n🔍 Testing configuration...");
    test_configuration(&blockfrost_config, &pools).await?;

    // Save configuration
    save_configuration(&blockfrost_config, &pools)?;

    println!("\n✅ Setup complete! You can now run:");
    println!("   cargo run -- monitor");
    println!("   cargo run -- health-check");

    Ok(())
}

async fn setup_blockfrost() -> Result<BlockfrostConfig> {
    println!("🔑 Setting up Blockfrost API...");
    println!("Visit https://blockfrost.io to get a free API key\n");

    let network = prompt_choice(
        "Select your network",
        &["mainnet", "testnet", "preprod"],
        "mainnet"
    )?;

    let api_key = prompt(&format!(
        "Enter your Blockfrost API key for {} (starts with '{}'): ",
        network,
        network
    ))?;

    if !validate_blockfrost_key(&api_key, &network) {
        anyhow::bail!("Invalid Blockfrost API key format. Should start with '{}'", network);
    }

    let base_url = match network.as_str() {
        "mainnet" => "https://cardano-mainnet.blockfrost.io/api/v0",
        "testnet" => "https://cardano-testnet.blockfrost.io/api/v0",
        "preprod" => "https://cardano-preprod.blockfrost.io/api/v0",
        _ => anyhow::bail!("Invalid network"),
    };

    Ok(BlockfrostConfig {
        api_key,
        base_url: base_url.to_string(),
    })
}

async fn setup_pools() -> Result<Vec<PoolConfig>> {
    println!("\n🏊 Setting up your stake pools...");

    let mut pools = Vec::new();

    loop {
        println!("\nAdding pool #{}", pools.len() + 1);

        let pool_id = prompt("Enter your stake pool ID (pool1...): ")?;
        if !validate_pool_id(&pool_id) {
            println!("❌ Invalid pool ID format. Should start with 'pool1' and be 56 characters long.");
            continue;
        }

        let name = prompt("Enter pool name: ")?;
        let ticker = prompt("Enter pool ticker: ")?;

        // Required fields - use defaults for now
        let vrf_key_file = prompt_with_default(
            "VRF key file path",
            "/opt/cardano/keys/vrf.vkey"
        )?;

        let pledge_address = prompt_with_default(
            "Pledge address",
            "addr1..."
        )?;

        let reward_address = prompt_with_default(
            "Reward address",
            "stake1..."
        )?;

        pools.push(PoolConfig {
            pool_id,
            name,
            ticker,
            vrf_key_file,
            pledge_address,
            reward_address,
        });

        if !prompt_yes_no("Add another pool?", false)? {
            break;
        }
    }

    if pools.is_empty() {
        anyhow::bail!("At least one pool is required");
    }

    Ok(pools)
}

async fn test_configuration(blockfrost_config: &BlockfrostConfig, pools: &[PoolConfig]) -> Result<()> {
    // Create a temporary config for testing
    let test_config = Config {
        cardano: CardanoConfig {
            cli_path: "cardano-cli".to_string(),
            node_socket_path: "/tmp/socket".to_string(),
            network: "mainnet".to_string(),
            testnet_magic: None,
        },
        blockfrost: Some(blockfrost_config.clone()),
        pools: pools.to_vec(),
        monitoring: MonitoringConfig {
            enabled: true,
            prometheus_port: 9090,
            check_interval_seconds: 30,
            metrics_interval: 30, // Add missing field
            alerts: AlertsConfig {
                saturation_threshold: 0.8,
                missed_blocks_threshold: 2,
                email_enabled: false,
                webhook_url: "".to_string(),
            },
        },
        rewards: RewardsConfig {
            calculation_method: "standard".to_string(), // Add missing field
            output_format: "json".to_string(),          // Add missing field
            output_directory: "./reports".to_string(),
            include_fees: true,
            delegation_rewards_percentage: 95.0,
        },
    };

    // Test Blockfrost connection
    print!("  ✓ Testing Blockfrost API connection... ");
    io::stdout().flush()?;

    if let Some(client) = BlockfrostClient::new(&test_config) {
        match client.get_network_info().await {
            Ok(_) => println!("✅ Success!"),
            Err(e) => {
                println!("❌ Failed: {}", e);
                anyhow::bail!("Blockfrost API test failed. Please check your API key.");
            }
        }
    } else {
        anyhow::bail!("Failed to create Blockfrost client");
    }

    // Test pool existence
    for pool in pools {
        print!("  ✓ Testing pool {} ({})... ", pool.ticker, pool.pool_id);
        io::stdout().flush()?;

        if let Some(client) = BlockfrostClient::new(&test_config) {
            match client.get_pool_info(&pool.pool_id).await {
                Ok(_) => println!("✅ Found!"),
                Err(_) => {
                    println!("⚠️  Pool not found or inactive");
                    if !prompt_yes_no("Continue anyway?", true)? {
                        anyhow::bail!("Setup cancelled");
                    }
                }
            }
        }
    }

    Ok(())
}

fn save_configuration(blockfrost_config: &BlockfrostConfig, pools: &[PoolConfig]) -> Result<()> {
    let config = Config {
        cardano: CardanoConfig {
            cli_path: "cardano-cli".to_string(),
            node_socket_path: "/opt/cardano/cnode/sockets/node0.socket".to_string(),
            network: "mainnet".to_string(),
            testnet_magic: None,
        },
        blockfrost: Some(blockfrost_config.clone()),
        pools: pools.to_vec(),
        monitoring: MonitoringConfig {
            enabled: true,
            prometheus_port: 9090,
            check_interval_seconds: 30,
            metrics_interval: 30, // Add missing field
            alerts: AlertsConfig {
                saturation_threshold: 0.8,
                missed_blocks_threshold: 2,
                email_enabled: false,
                webhook_url: "".to_string(),
            },
        },
        rewards: RewardsConfig {
            calculation_method: "standard".to_string(), // Add missing field
            output_format: "json".to_string(),          // Add missing field
            output_directory: "./reports".to_string(),
            include_fees: true,
            delegation_rewards_percentage: 95.0,
        },
    };

    let config_path = get_config_path();
    let config_toml = toml::to_string_pretty(&config)?;

    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&config_path, config_toml)?;

    println!("💾 Configuration saved to: {}", config_path.display());

    Ok(())
}

fn get_config_path() -> PathBuf {
    PathBuf::from("config.toml")
}

// Helper functions for user input
fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_with_default(message: &str, default: &str) -> Result<String> {
    let input = prompt(&format!("{} [{}]: ", message, default))?;
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input)
    }
}

fn prompt_choice(message: &str, choices: &[&str], default: &str) -> Result<String> {
    println!("{}", message);
    for (i, choice) in choices.iter().enumerate() {
        let marker = if *choice == default { " (default)" } else { "" };
        println!("  {}. {}{}", i + 1, choice, marker);
    }

    let input = prompt("Enter choice [1]: ")?;
    if input.is_empty() {
        Ok(default.to_string())
    } else {
        let index: usize = input.parse::<usize>()? - 1;
        if index < choices.len() {
            Ok(choices[index].to_string())
        } else {
            anyhow::bail!("Invalid choice")
        }
    }
}

fn prompt_yes_no(message: &str, default: bool) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    let input = prompt(&format!("{} [{}]: ", message, default_str))?;

    if input.is_empty() {
        Ok(default)
    } else {
        match input.to_lowercase().as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => {
                println!("Please enter 'y' or 'n'");
                prompt_yes_no(message, default)
            }
        }
    }
}

fn validate_pool_id(pool_id: &str) -> bool {
    pool_id.starts_with("pool1") && pool_id.len() == 56
}

fn validate_blockfrost_key(key: &str, network: &str) -> bool {
    key.len() > 10 && key.starts_with(network)
}