use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub cardano: CardanoConfig,
    pub pools: Vec<PoolConfig>,
    pub monitoring: MonitoringConfig,
    pub rewards: RewardsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CardanoConfig {
    pub cli_path: String,
    pub node_socket_path: String,
    pub network: String,
    pub testnet_magic: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PoolConfig {
    pub pool_id: String,
    pub name: String,
    pub ticker: String,
    pub vrf_key_file: String,
    pub pledge_address: String,
    pub reward_address: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub prometheus_port: u16,
    pub metrics_interval: u64,
    pub check_interval_seconds: u64,
    pub alerts: AlertsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AlertsConfig {
    pub email_enabled: bool,
    pub webhook_url: String,
    pub saturation_threshold: f64,
    pub missed_blocks_threshold: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RewardsConfig {
    pub calculation_method: String,
    pub output_format: String,
    pub output_directory: String,
    pub include_fees: bool,
    pub delegation_rewards_percentage: f64,
}

impl Config {
    pub fn load_or_create_default() -> Result<Self> {
        let config_path = "config.toml";

        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let default_config = Self::default();
            let toml_content = toml::to_string_pretty(&default_config)?;
            fs::write(config_path, toml_content)?;
            println!("Created default config file: {}", config_path);
            Ok(default_config)
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cardano: CardanoConfig {
                cli_path: "cardano-cli".to_string(),
                node_socket_path: "/opt/cardano/cnode/sockets/node0.socket".to_string(),
                network: "mainnet".to_string(),
                testnet_magic: None,
            },
            pools: vec![
                PoolConfig {
                    pool_id: "pool1abc123...".to_string(),
                    name: "My Stake Pool".to_string(),
                    ticker: "DEMO".to_string(),
                    vrf_key_file: "vrf.vkey".to_string(),
                    pledge_address: "addr1...".to_string(),
                    reward_address: "stake1...".to_string(),
                },
                PoolConfig {
                    pool_id: "pool1def456...".to_string(),
                    name: "Backup Pool".to_string(),
                    ticker: "BACKUP".to_string(),
                    vrf_key_file: "backup_vrf.vkey".to_string(),
                    pledge_address: "addr1...".to_string(),
                    reward_address: "stake1...".to_string(),
                },
            ],
            monitoring: MonitoringConfig {
                enabled: true,
                prometheus_port: 9090,
                metrics_interval: 60,
                check_interval_seconds: 30,
                alerts: AlertsConfig {
                    email_enabled: false,
                    webhook_url: "".to_string(),
                    saturation_threshold: 0.8,
                    missed_blocks_threshold: 2,
                },
            },
            rewards: RewardsConfig {
                calculation_method: "standard".to_string(),
                output_format: "json".to_string(),
                output_directory: "./reports".to_string(),
                include_fees: true,
                delegation_rewards_percentage: 95.0,
            },
        }
    }
}