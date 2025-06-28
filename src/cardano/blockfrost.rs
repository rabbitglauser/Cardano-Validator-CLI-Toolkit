use anyhow::{Result, Context};
use reqwest::Client;
use serde_json::Value;
use crate::utils::config::Config;

pub struct BlockfrostClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl BlockfrostClient {
    pub fn new(config: &Config) -> Option<Self> {
        match &config.blockfrost {
            Some(blockfrost_config) => {
                Some(Self {
                    client: Client::new(),
                    base_url: blockfrost_config.base_url.clone(),
                    api_key: blockfrost_config.api_key.clone(),
                })
            }
            None => None,
        }
    }

    pub async fn get_network_info(&self) -> Result<Value> {
        let url = format!("{}/network", self.base_url);

        let response = self.client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to send request to Blockfrost API")?;

        if !response.status().is_success() {
            anyhow::bail!("Blockfrost API returned status: {}", response.status());
        }

        let json: Value = response.json()
            .await
            .context("Failed to parse JSON response from Blockfrost")?;

        Ok(json)
    }

    pub async fn get_latest_epoch(&self) -> Result<Value> {
        let url = format!("{}/epochs/latest", self.base_url);

        let response = self.client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to send request to Blockfrost API")?;

        if !response.status().is_success() {
            anyhow::bail!("Blockfrost API returned status: {}", response.status());
        }

        let json: Value = response.json()
            .await
            .context("Failed to parse JSON response from Blockfrost")?;

        Ok(json)
    }

    pub async fn get_all_pools(&self) -> Result<Value> {
        let url = format!("{}/pools", self.base_url);

        let response = self.client
            .get(&url)
            .header("project_id", &self.api_key)
            .query(&[("count", "100")])
            .send()
            .await
            .context("Failed to send request to Blockfrost API")?;

        if !response.status().is_success() {
            anyhow::bail!("Blockfrost API returned status: {}", response.status());
        }

        let json: Value = response.json()
            .await
            .context("Failed to parse JSON response from Blockfrost")?;

        Ok(json)
    }

    pub async fn get_pool_info(&self, pool_id: &str) -> Result<Value> {
        let url = format!("{}/pools/{}", self.base_url, pool_id);

        let response = self.client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to send request to Blockfrost API")?;

        if !response.status().is_success() {
            anyhow::bail!("Blockfrost API returned status: {}", response.status());
        }

        let json: Value = response.json()
            .await
            .context("Failed to parse JSON response from Blockfrost")?;

        Ok(json)
    }

    pub async fn get_pool_metadata(&self, pool_id: &str) -> Result<Value> {
        let url = format!("{}/pools/{}/metadata", self.base_url, pool_id);

        let response = self.client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to send request to Blockfrost API")?;

        if !response.status().is_success() {
            anyhow::bail!("Blockfrost API returned status: {}", response.status());
        }

        let json: Value = response.json()
            .await
            .context("Failed to parse JSON response from Blockfrost")?;

        Ok(json)
    }
}