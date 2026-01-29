# Ubuntu Resource API

A RESTful web API written in Rust for monitoring Ubuntu system resources.

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

## 🎨 Dashboard

Open your browser and navigate to **`http://127.0.0.1:8080/dashboard`** for a beautiful, real-time system monitoring dashboard featuring:

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
