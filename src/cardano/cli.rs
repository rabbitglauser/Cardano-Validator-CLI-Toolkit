use anyhow::{Result, Context};
use serde_json::Value;
use std::process::Command;
use crate::utils::config::Config;

pub struct CardanoCli {
    cli_path: String,
    socket_path: String,
    network: String,
}

impl CardanoCli {
    pub fn new(config: &Config) -> Self {
        Self {
            cli_path: config.cardano.cli_path.clone(),
            socket_path: config.cardano.node_socket_path.clone(),
            network: config.cardano.network.clone(),
        }
    }

    pub async fn query_tip(&self) -> Result<Value> {
        let output = Command::new(&self.cli_path)
            .args([
                "query", "tip",
                "--socket-path", &self.socket_path,
                &format!("--{}", self.network),
            ])
            .output()
            .context("Failed to execute cardano-cli query tip")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cardano-cli query tip failed: {}", error);
        }

        let result = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in cardano-cli output")?;

        serde_json::from_str(&result)
            .context("Failed to parse JSON response from cardano-cli")
    }

    pub async fn query_stake_distribution(&self) -> Result<Value> {
        let output = Command::new(&self.cli_path)
            .args([
                "query", "stake-distribution",
                "--socket-path", &self.socket_path,
                &format!("--{}", self.network),
            ])
            .output()
            .context("Failed to execute cardano-cli query stake-distribution")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cardano-cli query stake-distribution failed: {}", error);
        }

        let result = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in cardano-cli output")?;

        serde_json::from_str(&result)
            .context("Failed to parse JSON response from cardano-cli")
    }

    pub async fn query_pool_params(&self, pool_id: &str) -> Result<Value> {
        let output = Command::new(&self.cli_path)
            .args([
                "query", "pool-params",
                "--stake-pool-id", pool_id,
                "--socket-path", &self.socket_path,
                &format!("--{}", self.network),
            ])
            .output()
            .context("Failed to execute cardano-cli query pool-params")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cardano-cli query pool-params failed: {}", error);
        }

        let result = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in cardano-cli output")?;

        serde_json::from_str(&result)
            .context("Failed to parse JSON response from cardano-cli")
    }

    pub async fn query_pool_info(&self, pool_id: &str) -> Result<Value> {
        // First try to get pool params to see if pool exists and is active
        match self.query_pool_params(pool_id).await {
            Ok(params) => {
                // If we get params, the pool is active
                Ok(serde_json::json!({
                    "active": true,
                    "params": params
                }))
            }
            Err(_) => {
                // Pool might be retired or doesn't exist
                Ok(serde_json::json!({
                    "active": false,
                    "params": null
                }))
            }
        }
    }

    pub async fn query_pool_blocks(&self, _pool_id: &str, _epoch: u64) -> Result<Value> {
        //TODO This is a simplified approach. In reality, you'd need to:
        // 1. Query the ledger state for the epoch
        // 2. Parse block producer information
        // For now, we'll return a placeholder that indicates "not implemented"

        // Try to get ledger state (this is expensive and might fail)
        match self.query_ledger_state().await {
            Ok(_ledger) => {
                // TODO: Parse actual block information from ledger state
                // This would require complex parsing of the ledger state JSON
                Ok(serde_json::json!([]))  // Return empty array for now
            }
            Err(_) => {
                // Return empty array if we can't query ledger state
                Ok(serde_json::json!([]))
            }
        }
    }

    pub async fn query_ledger_state(&self) -> Result<Value> {
        let output = Command::new(&self.cli_path)
            .args([
                "query", "ledger-state",
                "--socket-path", &self.socket_path,
                &format!("--{}", self.network),
            ])
            .output()
            .context("Failed to execute cardano-cli query ledger-state")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cardano-cli query ledger-state failed: {}", error);
        }

        let result = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in cardano-cli output")?;

        serde_json::from_str(&result)
            .context("Failed to parse JSON response from cardano-cli")
    }

    // Helper method to check if cardano-cli is available
    pub async fn is_available(&self) -> bool {
        let output = Command::new(&self.cli_path)
            .args(&["version"])
            .output();

        match output {
            Ok(result) => result.status.success(),
            Err(_) => false,
        }
    }

    pub async fn query_stake_pools(&self) -> Result<Vec<String>> {
        let output = Command::new(&self.cli_path)
            .args([
                "query", "stake-pools",
                "--socket-path", &self.socket_path,
                &format!("--{}", self.network),
            ])
            .output()
            .context("Failed to execute cardano-cli query stake-pools")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cardano-cli query stake-pools failed: {}", error);
        }

        let result = String::from_utf8(output.stdout)?;
        let pools: Vec<String> = serde_json::from_str(&result)?;
        Ok(pools)
    }

    pub async fn query_leadership_schedule(&self, pool_id: &str, vrf_key_file: &str) -> Result<Value> {
        let output = Command::new(&self.cli_path)
            .args([
                "query", "leadership-schedule",
                "--stake-pool-id", pool_id,
                "--vrf-signing-key-file", vrf_key_file,
                "--socket-path", &self.socket_path,
                &format!("--{}", self.network),
            ])
            .output()
            .context("Failed to execute cardano-cli query leadership-schedule")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cardano-cli query leadership-schedule failed: {}", error);
        }

        let result = String::from_utf8(output.stdout)?;
        serde_json::from_str(&result).context("Failed to parse leadership schedule")
    }
}