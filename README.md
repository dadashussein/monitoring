# Ubuntu Resource Monitor

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange?logo=rust)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

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
- 🔄 **Nginx Proxy Manager** - Create and manage reverse proxies
- 🐳 **Docker Manager** - Manage containers, images, volumes, and networks

## 📦 Installation Methods

| Method | Rust Required | Build Time | Best For |
|--------|---------------|------------|----------|
| **Pre-built Binary** | ❌ No | ⚡ Instant | Production servers |
| **Build from Source** | ✅ Yes | 🔨 ~5 min | Development |

## 🚀 Quick Start

> 📖 **Detailed installation guide**: See [INSTALL.md](INSTALL.md) for complete installation instructions.

### Recommended: One-Command Install (No Rust Required!) ⭐

Download and install pre-built binary:

```bash
curl -sSL https://raw.githubusercontent.com/YOUR_USERNAME/ubuntu-resource-monitor/main/install-binary.sh | sudo bash
```

Or download and run manually:

```bash
wget https://raw.githubusercontent.com/YOUR_USERNAME/ubuntu-resource-monitor/main/install-binary.sh
chmod +x install-binary.sh
sudo ./install-binary.sh
```

**What this does:**
- ✅ Downloads pre-built binary (no compilation needed!)
- ✅ Asks for your preferred port and address
- ✅ Installs to `/opt/ubuntu-resource-monitor`
- ✅ Creates systemd service
- ✅ Starts automatically

**No Rust, no Cargo, no Docker - just works!** 🎉

**Service Management:**
```bash
# Check status
sudo systemctl status ubuntu-resource-monitor

# View logs
sudo journalctl -u ubuntu-resource-monitor -f

# Restart service
sudo systemctl restart ubuntu-resource-monitor

# Stop service
sudo systemctl stop ubuntu-resource-monitor

# Uninstall
sudo bash uninstall.sh
```

### Alternative: Build from Source

If you want to build from source:

### Option 2: Install from Source (If you want to build yourself)

### Alternative: Build from Source

If you want to build from source:

```bash
# Clone repository
git clone <repository-url>
cd ubuntu-resource-monitor

# Run installer (will build from source)
sudo bash install.sh
```

The installer will:
- ✅ Install Rust/Cargo if needed
- ✅ Build the application
- ✅ Install and configure systemd service

## 🔧 Manual Build and Run

## 🔧 Manual Build and Run

For development or testing:

```bash
# Build
cargo build --release

# Run with custom port and address
SERVER_BIND_ADDRESS="0.0.0.0:9000" cargo run --release

# Or run the binary directly
./target/release/ubuntu_resource_api
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

## ⚙️ Configuration

The application can be configured using environment variables. All settings have sensible defaults.

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_BIND_ADDRESS` | Server bind address and port | `0.0.0.0:8080` |
| `NGINX_SITES_AVAILABLE` | Nginx sites-available directory | `/etc/nginx/sites-available` |
| `NGINX_SITES_ENABLED` | Nginx sites-enabled directory | `/etc/nginx/sites-enabled` |
| `DOCKER_SOCKET_PATH` | Docker socket path | `unix:///var/run/docker.sock` |

### Example Configuration

```bash
# Custom server port
export SERVER_BIND_ADDRESS="127.0.0.1:9000"

# Custom nginx paths
export NGINX_SITES_AVAILABLE="/custom/nginx/available"
export NGINX_SITES_ENABLED="/custom/nginx/enabled"

# Remote Docker daemon
export DOCKER_SOCKET_PATH="tcp://localhost:2375"

# Run the application
cargo run --release
```

### Docker Configuration

When using Docker Compose, set environment variables in `docker-compose.yml`:

```yaml
environment:
  - SERVER_BIND_ADDRESS=0.0.0.0:8080
  - NGINX_SITES_AVAILABLE=/etc/nginx/sites-available
  - NGINX_SITES_ENABLED=/etc/nginx/sites-enabled
  - DOCKER_SOCKET_PATH=unix:///var/run/docker.sock
```

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
- **Serde** - Serialization
- **Chrono** - Date/time handling
- **Bollard** - Docker API client

## 🏗️ Architecture

The application follows a modular architecture with clear separation of concerns:

### Module Design

Each feature module (system, nginx, docker) follows a consistent structure:

- **models.rs**: Data structures and domain entities
- **handlers.rs**: HTTP request handlers and business logic
- **routes.rs**: Route registration and API endpoint configuration
- **Additional utilities**: Module-specific helper functions (e.g., nginx/config.rs, docker/client.rs)

### Shared Infrastructure

- **config.rs**: Centralized configuration management with environment variable support
- **error.rs**: Consistent error handling with automatic HTTP response conversion
- **utils.rs**: Shared utility functions used across modules

### Request Flow

1. HTTP request arrives at the server
2. Actix-web routes the request to the appropriate module handler
3. Handler processes the request using module-specific logic
4. Errors are automatically converted to consistent JSON responses
5. Response is returned to the client

This architecture makes the codebase:
- **Maintainable**: Each module is self-contained and focused
- **Testable**: Modules can be tested in isolation
- **Extensible**: New features can be added as new modules following the same pattern

## 📦 Project Structure

The application follows a modular architecture with clear separation of concerns:

```
.
├── src/
│   ├── main.rs              # Application entry point and server initialization
│   ├── lib.rs               # Library root exposing public modules
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Common error types and handling
│   ├── utils.rs             # Shared utility functions
│   ├── system/              # System monitoring module
│   │   ├── mod.rs          # Module declaration
│   │   ├── models.rs       # System info data structures
│   │   ├── handlers.rs     # HTTP request handlers
│   │   └── routes.rs       # Route registration
│   ├── nginx/               # Nginx management module
│   │   ├── mod.rs          # Module declaration
│   │   ├── models.rs       # Nginx proxy data structures
│   │   ├── handlers.rs     # HTTP request handlers
│   │   ├── config.rs       # Nginx config generation and validation
│   │   └── routes.rs       # Route registration
│   ├── docker/              # Docker management module
│   │   ├── mod.rs          # Module declaration
│   │   ├── models.rs       # Docker entity data structures
│   │   ├── handlers.rs     # HTTP request handlers
│   │   ├── client.rs       # Docker client utilities
│   │   └── routes.rs       # Route registration
│   └── templates/           # HTML dashboard files
│       ├── dashboard.html
│       ├── nginx_admin.html
│       └── docker_manager.html
├── Dockerfile               # Production build
├── Dockerfile.dev           # Development with hot reload
├── docker-compose.yml       # Docker Compose configuration
├── Makefile                # Build commands
└── Cargo.toml              # Rust dependencies
```

### Module Organization

- **system/**: System monitoring functionality (CPU, memory, disk, network, processes)
- **nginx/**: Nginx proxy management (CRUD operations, config generation)
- **docker/**: Docker management (containers, images, volumes, networks)
- **config.rs**: Centralized configuration with environment variable support
- **error.rs**: Consistent error handling across all modules
- **utils.rs**: Shared utility functions (e.g., unit conversions)

## 🧹 Cleanup

```bash
# Clean build artifacts
make clean

# Clean Docker resources
make docker-clean
```

## 👨‍💻 Development Guide

### Adding a New Feature Module

To add a new feature module, follow the established pattern:

1. Create a new directory under `src/` (e.g., `src/myfeature/`)
2. Create the module structure:
   ```
   src/myfeature/
   ├── mod.rs       # Module declaration with documentation
   ├── models.rs    # Data structures
   ├── handlers.rs  # HTTP handlers
   └── routes.rs    # Route registration
   ```
3. Add module documentation in `mod.rs`
4. Implement your models, handlers, and routes
5. Export the module in `src/lib.rs`
6. Register routes in `src/main.rs`

### Code Style

- Use Rust standard formatting: `cargo fmt`
- Check for linting issues: `cargo clippy`
- Run tests before committing: `cargo test`
- Generate documentation: `cargo doc --open`

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Generate test coverage
cargo tarpaulin
```

## 📄 License

This project is open source and available under the MIT License.
