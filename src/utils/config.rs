use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub cardano: CardanoConfig,
    pub pools: Vec<PoolConfig>,
    pub monitoring: MonitoringConfig,
    pub rewards: RewardsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CardanoConfig {
    pub cli_path: String,
    pub node_socket_path: String,
    pub network: String, // mainnet, testnet, preprod
    pub magic: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoolConfig {
    pub pool_id: String,
    pub name: String,
    pub vrf_key_file: String,
    pub cold_key_file: String,
    pub cert_file: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub prometheus_port: u16,
    pub check_interval_seconds: u64,
    pub alerts: AlertConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AlertConfig {
    pub saturation_threshold: f64,
    pub sync_lag_threshold_seconds: u64,
    pub missed_blocks_threshold: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RewardsConfig {
    pub default_format: String,
    pub output_directory: String,
    pub include_fees: bool,
}

pub fn load_config(path: &str) -> Result<Config> {
    let config_path = Path::new(path);

    if !config_path.exists() {
        return Err(anyhow::anyhow!("Configuration file not found: {}", path));
    }

    let config_str = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;

    Ok(config)
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cardano: CardanoConfig {
                cli_path: "cardano-cli".to_string(),
                node_socket_path: "/opt/cardano/cnode/sockets/node0.socket".to_string(),
                network: "mainnet".to_string(),
                magic: None,
            },
            pools: vec![],
            monitoring: MonitoringConfig {
                enabled: true,
                prometheus_port: 9090,
                check_interval_seconds: 300,
                alerts: AlertConfig {
                    saturation_threshold: 0.95,
                    sync_lag_threshold_seconds: 120,
                    missed_blocks_threshold: 3,
                },
            },
            rewards: RewardsConfig {
                default_format: "table".to_string(),
                output_directory: "./reports".to_string(),
                include_fees: true,
            },
        }
    }
}