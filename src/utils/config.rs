
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;

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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PoolConfig {
    pub pool_id: String,
    pub name: String,
    pub ticker: String,
    pub vrf_key_file: String,
    pub cold_key_file: String,
    pub cert_file: String,
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
    pub saturation_threshold: f64,
    pub sync_lag_threshold_seconds: u64,
    pub missed_blocks_threshold: u64,
    pub webhook_url: String,
    pub email_enabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RewardsConfig {
    pub default_format: String,
    pub output_directory: String,
    pub include_fees: bool,
}

pub fn load_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path))?;

    let config: Config = toml::from_str(&content)
        .context("Failed to parse TOML config")?;

    Ok(config)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cardano: CardanoConfig {
                cli_path: "cardano-cli".to_string(),
                node_socket_path: "/opt/cardano/cnode/sockets/node.socket".to_string(),
                network: "mainnet".to_string(),
            },
            pools: vec![
                PoolConfig {
                    pool_id: "pool1abcdef123456789abcdef123456789abcdef123456789abcdef123456789".to_string(),
                    name: "My Stake Pool".to_string(),
                    ticker: "MYPOOL".to_string(),
                    vrf_key_file: "/opt/cardano/cnode/priv/pool/vrf.skey".to_string(),
                    cold_key_file: "/opt/cardano/cnode/priv/pool/cold.skey".to_string(),
                    cert_file: "/opt/cardano/cnode/priv/pool/pool.cert".to_string(),
                }
            ],
            monitoring: MonitoringConfig {
                enabled: true,
                prometheus_port: 9090,
                metrics_interval: 60,
                check_interval_seconds: 300,
                alerts: AlertsConfig {
                    saturation_threshold: 0.95,
                    sync_lag_threshold_seconds: 120,
                    missed_blocks_threshold: 3,
                    webhook_url: String::new(),
                    email_enabled: false,
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