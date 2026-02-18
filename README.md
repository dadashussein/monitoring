# Ubuntu Resource Monitor

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange?logo=rust)](https://rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue?logo=docker)](https://docker.com)

Local desktop system monitoring tool with real-time dashboard. Monitor CPU, memory, disk, network and processes on your Linux machine. Kill processes directly from the dashboard.

## ✨ Features

- 🖥️ System information (hostname, OS, kernel version, uptime)
- 💻 CPU information and usage statistics (per-core visualization)
- 🧠 Memory usage (total, used, free, available)
- 💾 Disk usage for all mounted filesystems
- 🌐 Network interface statistics
- 🔧 Running processes with CPU/memory usage
- ✕ **Kill processes** directly from dashboard
- 📊 Sort processes by CPU or Memory usage
- ⚡ System load average
- 🎨 Beautiful dark-themed dashboard with real-time updates

## 🚀 Quick Start with Docker (Recommended)

### Development Mode (Hot Reload)

```bash
# Start development server with hot reload
make dev

# View logs
make dev-logs

# Stop server
make dev-stop
```

The server will start at `http://localhost:8080` and automatically reload when you change the code.

### Production Mode

```bash
# Build and start production server
make prod

# View logs
make prod-logs

# Stop server
make prod-stop
```

### All Docker Commands

```bash
make help              # Show all available commands
make dev               # Start development server
make dev-build         # Build development image
make dev-logs          # Show development logs
make dev-stop          # Stop development server
make prod              # Start production server
make prod-build        # Build production image
make prod-logs         # Show production logs
make prod-stop         # Stop production server
make docker-clean      # Clean Docker resources
```

## 🔧 Alternative: Build Locally

### With Docker (No Rust Installation Required)

```bash
# Build binary using Docker
make build

# Run the application
make run
```

### With Rust

```bash
# Build
cargo build --release

# Run
cargo run --release
```

## 📡 API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | API information and available endpoints |
| `GET` | `/dashboard` | 🎨 **Web Dashboard** - Modern real-time system monitor |
| `GET` | `/api/system` | System information |
| `GET` | `/api/cpu` | CPU information |
| `GET` | `/api/cpu/usage` | CPU usage statistics |
| `GET` | `/api/memory` | Memory usage information |
| `GET` | `/api/disks` | Disk usage for all mounts |
| `GET` | `/api/network` | Network interface statistics |
| `GET` | `/api/processes?limit=N` | Top N processes (default: 50) |
| `GET` | `/api/load` | System load average (1, 5, 15 min) |
| `DELETE` | `/api/processes/:pid` | Kill process by PID |
| `GET` | `/health` | Health check |

## 🎨 Dashboard

Open your browser and navigate to **`http://localhost:8080/dashboard`** for a beautiful, real-time system monitoring dashboard featuring:

- **Dark theme** with gradient accents and modern design
- **Real-time updates** (auto-refresh every 3 seconds)
- **CPU monitoring** with per-core usage visualization
- **Memory usage** with detailed statistics
- **Disk usage** for all mounted filesystems
- **Network interfaces** with traffic statistics
- **Top processes** sorted by CPU or Memory usage
- **Kill processes** with X button (confirmation required)
- **System load average** tracking
- **System info** including uptime, OS, and kernel version

## 📝 Example Usage

```bash
# Get system info
curl http://localhost:8080/api/system

# Get CPU usage
curl http://localhost:8080/api/cpu/usage

# Get memory info
curl http://localhost:8080/api/memory

# Get top 10 processes
curl http://localhost:8080/api/processes?limit=10

# Kill a process
curl -X DELETE http://localhost:8080/api/processes/1234
```

## 🛠️ Technologies

- **Rust 1.83** - Systems programming language
- **Actix-web** - Fast web framework
- **Sysinfo** - System information gathering
- **Docker & Docker Compose** - Containerization
- **Serde** - Serialization
- **Chrono** - Date/time handling

## 📦 Project Structure

```
.
├── src/
│   ├── main.rs          # Backend API
│   └── dashboard.html   # Frontend dashboard
├── Dockerfile           # Production build
├── Dockerfile.dev       # Development with hot reload
├── docker-compose.yml   # Docker Compose configuration
├── Makefile            # Build commands
└── Cargo.toml          # Rust dependencies
```

## 🧹 Cleanup

```bash
# Clean build artifacts
make clean

# Clean Docker resources
make docker-clean
```

## 📄 License

This project is open source and available under the MIT License.
