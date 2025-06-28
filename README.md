# 🛠️ Cardano Validator CLI Toolkit

A professional-grade command-line toolkit for Cardano Stake Pool Operators (SPOs), enabling powerful automation, health monitoring, analytics, rewards calculations, and node operations.

---

## 🚀 Overview

The **Cardano Validator CLI Toolkit** streamlines DevOps for SPOs by offering real-time monitoring, intelligent analytics, automated reward tracking, and efficient node management. Designed for reliability and performance, it's built with Rust and tailored for the Cardano blockchain ecosystem.

---

## ✨ Key Features

### 🩺 Health Monitoring
- Real-time pool health checks with rich metrics
- Saturation level monitoring with thresholds
- Block production tracking and alerts
- Node synchronization status
- Continuous background monitoring
- Alerts via webhooks or email

### 📊 Advanced Analytics
- Multi-epoch performance trends and insights
- ROI and pool efficiency calculations
- Delegator stats and growth tracking
- Compare multiple pools side-by-side
- Export analytics for external tools

### 💰 Rewards Management
- Automated reward breakdowns per epoch
- Delegator distribution and rewards analysis
- Historical rewards tracking and exporting
- Supports JSON exports and ROI metrics

### 🔧 Node Operations
- Manage node services: start, stop, restart
- Real-time node status diagnostics
- Configuration file validation
- Scheduled and automated maintenance

---

## 🧰 Tech Stack

- **Language**: Rust (2021 Edition)
- **CLI**: Clap v4 with derive macros
- **Async Runtime**: Tokio
- **Config**: Auto-generated TOML config
- **Output**: Rich CLI with colorized tables
- **Serialization**: Serde (JSON/TOML)
- **Error Handling**: Anyhow for robust flow control

---

## 📦 Installation

### Prerequisites
- Rust ≥ 1.70 with Cargo
- (Optional) Cardano node for live monitoring

### Build from Source
```bash
# Clone and enter project
git clone <repository-url>
cd cardano-validator-cli-toolkit

# Build release binary
cargo build --release

# (Optional) Install globally
cargo install --path .
⚡ Quick Start
Initial Setup
bash
Copy
Edit
# Generate default config
cargo run -- --help

# Then edit the generated `config.toml`
# → Set pool IDs, node paths, and monitoring settings
Basic Usage
bash
Copy
Edit
# Run health checks
cargo run -- health --all

# View analytics for last 10 epochs
cargo run -- analytics --detailed

# Calculate rewards for epoch 450
cargo run -- rewards --epoch 450 --detailed

# Check node status
cargo run -- node status
🧾 Commands Reference
🔍 Health Monitoring
bash
Copy
Edit
# One-time health check
cargo run -- health --all

# Continuous monitoring
cargo run -- health --all --watch --interval 30

# Export metrics
cargo run -- health --all --export ./reports
📈 Analytics & Reporting
bash
Copy
Edit
# Summary analytics
cargo run -- analytics

# Multi-epoch, detailed analytics
cargo run -- analytics --detailed --epochs 20

# Exported single-pool analysis
cargo run -- analytics --pool-id "pool1abc..." --detailed --export

# Custom epoch range
cargo run -- analytics --epochs 15 --export
💵 Rewards Calculation
bash
Copy
Edit
# Latest rewards
cargo run -- rewards --detailed

# Specific historical epoch
cargo run -- rewards --epoch 445 --detailed

# Batch processing for all pools
cargo run -- rewards --detailed
🖥️ Node Management
bash
Copy
Edit
# Status check
cargo run -- node status

# Manage service
cargo run -- node start
cargo run -- node stop
cargo run -- node restart
⚙️ Configuration File
The CLI generates a config.toml file on first run. Example:

toml
Copy
Edit
[cardano]
cli_path = "cardano-cli"
node_socket_path = "/opt/cardano/cnode/sockets/node0.socket"
network = "mainnet"

[[pools]]
pool_id = "pool1abc123..."
name = "My Stake Pool"
ticker = "DEMO"
vrf_key_file = "vrf.vkey"
pledge_address = "addr1..."
reward_address = "stake1..."

[monitoring]
enabled = true
prometheus_port = 9090
check_interval_seconds = 30

[monitoring.alerts]
saturation_threshold = 0.8
missed_blocks_threshold = 2
email_enabled = false
webhook_url = ""

[rewards]
output_directory = "./reports"
include_fees = true
delegation_rewards_percentage = 95.0
📊 Output Samples
✅ Health Check
sql
Copy
Edit
🏥 Health Check System

┌─────────────┬────────────┬─────────────┬──────────────┬─────────┬──────────┬────────┐
│ Pool        │ Status     │ Sync        │ Saturation   │ Blocks  │ Response │ Issues │
├─────────────┼────────────┼─────────────┼──────────────┼─────────┼──────────┼────────┤
│ My Pool     │ 🟢 Healthy │ 🟢 Synced   │ 🟢 4.2%      │ 🟢 3/4  │ 156ms    │ 0      │
│ Backup Pool │ 🟢 Healthy │ 🟢 Synced   │ 🟢 4.2%      │ 🟢 3/4  │ 142ms    │ 0      │
└─────────────┴────────────┴─────────────┴──────────────┴─────────┴──────────┴────────┘

✅ All pools are healthy!
📈 Analytics Report
yaml
Copy
Edit
📊 Advanced Analytics Report: My Stake Pool

Performance:
• Block Production: 95.0% 🟢
• Saturation Avg: 42.0%
• Reward Efficiency: 98.0%
• Uptime: 99.8%

Trend Analysis:
• Performance: 📈 Improving (+3.2%)
• Delegators: 📈 +8.5%
• Saturation: 📊 Stable
🔌 Integration & Use Cases
DevOps Integration
CI/CD: Trigger health checks during deploys

Monitoring: Export to Prometheus & Grafana

Alerts: Connect via webhook to Slack, Discord, PagerDuty

Reporting: Auto-generate stakeholder performance reports

Enterprise Use Cases
Manage multiple pools from one CLI

Track operational risks with smart alerts

Generate compliance and audit reports

Make data-driven decisions using pool analytics

🧭 Roadmap
✅ Phase 1: Core Complete
CLI, config management

Health monitoring and alerting

Analytics engine

Reward calculator

Node management

🚧 Phase 2: Integrations
 Live node data feeds

 Prometheus exporter

 Grafana dashboards

 Email alerting

 Webhook alerts

🧠 Phase 3: Advanced Features
 Persistent historical data

 ML-based trend forecasting

 Mainnet/Testnet multi-support

 API server mode

 Web dashboard GUI

🤝 Contributing
We follow best engineering practices:

Clean architecture & modular design

Comprehensive error handling

Unit + integration testing

Inline documentation

Optimized async I/O performance

📄 License
This project is licensed under the MIT License.

🙏 Acknowledgments
Cardano Foundation – for maintaining a solid blockchain backbone

Rust Lang Community – for powerful open-source tooling

SPO Contributors – for operational guidance and real-world needs

Built with ❤️ for the Cardano Ecosystem

yaml
Copy
Edit

---

Let me know if you'd like:
- A version with clickable badges (build status, version, etc.)
- Auto-generated GitHub Actions workflows
- A condensed “minimal” version for crates.io or package registries
- To localize it (e.g., Spanish, Japanese)

# list of all commands
cargo run -- --help

# seting everything up
cargo run -- setup

# Test health check
cargo run -- health-check

# Test pool status  
cargo run -- pool-status

# Test rewards calculation
cargo run -- rewards --detailed

# Test monitoring
cargo run -- monitor --prometheus --port 9090
