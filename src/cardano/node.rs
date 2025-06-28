use anyhow::Result;
use serde_json::Value;

pub struct CardanoNode {
    socket_path: String,
}

impl CardanoNode {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub async fn get_node_info(&self) -> Result<Value> {
        // TODO: Implement node info retrieval
        Ok(serde_json::json!({
            "status": "placeholder",
            "socket_path": self.socket_path
        }))
    }
}