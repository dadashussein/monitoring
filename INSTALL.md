# Installation Guide

## 🚀 Quick Installation (Recommended)

### Method 1: Pre-built Binary (No Rust Required!) ⭐

**One-command install:**

```bash
curl -sSL https://raw.githubusercontent.com/dadashussein/ubuntu-resource-monitor/main/install-binary.sh | sudo bash
```

Or download and run:

```bash
wget https://raw.githubusercontent.com/dadashussein/ubuntu-resource-monitor/main/install-binary.sh
chmod +x install-binary.sh
sudo ./install-binary.sh
```

**Advantages:**
- ✅ No Rust/Cargo installation needed
- ✅ No compilation time (instant install)
- ✅ Works on x86_64, ARM64, and ARMv7
- ✅ Smaller download size
- ✅ Perfect for production servers

### Method 2: Build from Source

**One-Command Install:**

```bash
curl -sSL https://raw.githubusercontent.com/dadashussein/ubuntu-resource-monitor/main/install.sh | sudo bash
```

Or download and run:

```bash
wget https://raw.githubusercontent.com/dadashussein/ubuntu-resource-monitor/main/install.sh
chmod +x install.sh
sudo ./install.sh
```

### What the installer does:

1. ✅ Checks system requirements
2. ✅ Installs Rust/Cargo if needed
3. ✅ Asks for your preferred port and address
4. ✅ Builds the application
5. ✅ Installs to `/opt/ubuntu-resource-monitor`
6. ✅ Creates systemd service
7. ✅ Starts the service automatically

### Interactive Configuration

During installation, you'll be asked:

```
Server adresi (default: 0.0.0.0): [Enter your IP or press Enter]
Server portu (default: 8080): [Enter your port or press Enter]
Nginx sites-available yolu (default: /etc/nginx/sites-available): [Press Enter]
Nginx sites-enabled yolu (default: /etc/nginx/sites-enabled): [Press Enter]
Docker socket yolu (default: unix:///var/run/docker.sock): [Press Enter]
```

## 📋 System Requirements

- **OS**: Ubuntu 20.04+ or Debian 11+
- **RAM**: 512MB minimum
- **Disk**: 100MB free space
- **Privileges**: Root access (sudo)

## 🔧 Service Management

After installation, manage the service with:

```bash
# Check status
sudo systemctl status ubuntu-resource-monitor

# Start service
sudo systemctl start ubuntu-resource-monitor

# Stop service
sudo systemctl stop ubuntu-resource-monitor

# Restart service
sudo systemctl restart ubuntu-resource-monitor

# View logs
sudo journalctl -u ubuntu-resource-monitor -f

# View last 100 lines
sudo journalctl -u ubuntu-resource-monitor -n 100
```

## 🌐 Access the Application

After installation, open your browser:

- **Dashboard**: `http://YOUR_IP:YOUR_PORT/dashboard`
- **Nginx Manager**: `http://YOUR_IP:YOUR_PORT/nginx`
- **Docker Manager**: `http://YOUR_IP:YOUR_PORT/docker`

Example: `http://192.168.1.100:8080/dashboard`

## 🔄 Update

To update to the latest version:

```bash
# Stop the service
sudo systemctl stop ubuntu-resource-monitor

# Pull latest code
cd /path/to/ubuntu-resource-monitor
git pull

# Rebuild
cargo build --release

# Copy new binary
sudo cp target/release/ubuntu_resource_api /opt/ubuntu-resource-monitor/

# Restart service
sudo systemctl start ubuntu-resource-monitor
```

## 🗑️ Uninstall

To completely remove the application:

```bash
sudo bash uninstall.sh
```

Or manually:

```bash
sudo systemctl stop ubuntu-resource-monitor
sudo systemctl disable ubuntu-resource-monitor
sudo rm /etc/systemd/system/ubuntu-resource-monitor.service
sudo rm -rf /opt/ubuntu-resource-monitor
sudo systemctl daemon-reload
```

## 🔐 Security Considerations

The application requires root privileges to:
- Monitor system processes
- Manage nginx configurations
- Access Docker socket
- Kill processes

**Recommendations:**
- Use firewall to restrict access to the web interface
- Change default port if exposed to internet
- Use reverse proxy with authentication for production
- Regularly update the application

## 🆘 Troubleshooting

### Service won't start

```bash
# Check logs
sudo journalctl -u ubuntu-resource-monitor -n 50

# Check if port is already in use
sudo netstat -tulpn | grep :8080

# Verify binary exists
ls -la /opt/ubuntu-resource-monitor/ubuntu_resource_api
```

### Can't access web interface

```bash
# Check if service is running
sudo systemctl status ubuntu-resource-monitor

# Check firewall
sudo ufw status
sudo ufw allow 8080/tcp

# Test locally
curl http://localhost:8080/health
```

### Permission errors

```bash
# Ensure service runs as root
sudo systemctl cat ubuntu-resource-monitor | grep User

# Check binary permissions
sudo chmod +x /opt/ubuntu-resource-monitor/ubuntu_resource_api
```

## 📞 Support

For issues and questions:
- Check logs: `sudo journalctl -u ubuntu-resource-monitor -f`
- Open an issue on GitHub
- Check README.md for configuration options
