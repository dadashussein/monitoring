# Ubuntu Resource API

[![Build](https://img.shields.io/badge/build-passing-success)](https://github.com/yourusername/ubuntu-resource-api)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange?logo=rust)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A **production-ready** RESTful web API written in Rust for monitoring Ubuntu system resources with a beautiful real-time dashboard.

## 🚀 One-Line Install

```bash
curl -sSL https://your-domain.com/install | sudo bash
```

Then open: `http://your-server-ip:8080/dashboard`

**No Docker. No dependencies. Just works.**

## Features

- System information (hostname, OS, kernel version, uptime)
- CPU information and usage statistics
- Memory usage (total, used, free, available)
- Disk usage for all mounted filesystems
- Network interface statistics
- Running processes with CPU/memory usage
- System load average

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /` | API information and available endpoints |
| `GET /dashboard` | 🎨 **Web Dashboard** - Modern real-time system monitor |
| `GET /api/system` | System information |
| `GET /api/cpu` | CPU information |
| `GET /api/cpu/usage` | CPU usage statistics |
| `GET /api/memory` | Memory usage information |
| `GET /api/disks` | Disk usage for all mounts |
| `GET /api/network` | Network interface statistics |
| `GET /api/processes?limit=N` | Top N processes by CPU usage (default: 50) |
| `GET /api/load` | System load average (1, 5, 15 min) |
| `GET /health` | Health check |

## Building

```bash
cd ubuntu-resource-api
cargo build --release
```

## Running

```bash
./target/release/ubuntu_resource_api
```

The server will start on `http://127.0.0.1:8080`

## 🚀 Deploying to Another Computer (No Rust Required!)

The binary is **portable** and runs on any modern Linux x86_64 system without installing Rust!

### Quick Deploy (One Command)

```bash
# Deploy to any Linux server
./deploy.sh user@remote-host

# Example
./deploy.sh ubuntu@192.168.1.100
```

### Manual Deploy

```bash
# 1. Copy the binary (only 6.7MB, includes dashboard)
scp target/release/ubuntu_resource_api user@remote-host:~/

# 2. On the remote machine, just run:
ssh user@remote-host
chmod +x ubuntu_resource_api
./ubuntu_resource_api

# 3. Open dashboard:
# http://remote-host:8080/dashboard
```

### Portable Package

A ready-to-deploy package is created for you:

```bash
# Extract and run on any Linux machine
tar -xzf ubuntu-resource-api-portable.tar.gz
./ubuntu_resource_api
```

**Requirements:** Linux x86_64 with glibc 2.31+ (Ubuntu 20.04+, Debian 11+, CentOS 8+, etc.)

### Run as Systemd Service

```bash
# Create service file
sudo tee /etc/systemd/system/ubuntu-resource-api.service > /dev/null <<EOF
[Unit]
Description=Ubuntu Resource API
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/ubuntu_resource_api
Restart=always
User=ubuntu

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo cp ubuntu_resource_api /usr/local/bin/
sudo systemctl daemon-reload
sudo systemctl enable ubuntu-resource-api
sudo systemctl start ubuntu-resource-api
```

## 🎨 Dashboard

Open your browser and navigate to **`http://127.0.0.1:8080/dashboard`** (or `http://<remote-ip>:8080/dashboard`) for a beautiful, real-time system monitoring dashboard featuring:

- **Dark theme** with gradient accents and modern design
- **Real-time updates** (auto-refresh every 3 seconds)
- **CPU monitoring** with per-core usage visualization
- **Memory usage** with detailed statistics
- **Disk usage** for all mounted filesystems
- **Network interfaces** with traffic statistics
- **Top processes** by CPU usage
- **System load average** tracking
- **System info** including uptime, OS, and kernel version

## Example Usage

```bash
# Get system info
curl http://127.0.0.1:8080/api/system

# Get CPU usage
curl http://127.0.0.1:8080/api/cpu/usage

# Get memory info
curl http://127.0.0.1:8080/api/memory

# Get top 10 processes
curl http://127.0.0.1:8080/api/processes?limit=10
```

## Dependencies

- [actix-web](https://actix.rs/) - Web framework
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - System information gathering
- [serde](https://serde.rs/) - Serialization
- [chrono](https://github.com/chronotope/chrono) - Date/time handling
