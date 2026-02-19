#!/bin/bash

# Ubuntu Resource Monitor - Binary Installation Script
# Downloads pre-built binary from GitHub Releases (no Rust/Cargo needed!)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO="YOUR_USERNAME/ubuntu-resource-monitor"  # Update this!
INSTALL_DIR="/opt/ubuntu-resource-monitor"
SERVICE_NAME="ubuntu-resource-monitor"
BINARY_NAME="ubuntu_resource_api"

# Default values
DEFAULT_PORT="8080"
DEFAULT_ADDRESS="0.0.0.0"

# Print colored message
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Check if running as root
check_root() {
    if [ "$EUID" -ne 0 ]; then 
        print_message "$RED" "❌ Bu skript root olaraq çalışdırılmalıdır. 'sudo' istifadə edin."
        exit 1
    fi
}

# Detect system architecture
detect_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        armv7l)
            ARCH="armv7"
            ;;
        *)
            print_message "$RED" "❌ Dəstəklənməyən arxitektura: $arch"
            exit 1
            ;;
    esac
    print_message "$GREEN" "✅ Arxitektura: $ARCH"
}

# Get latest release version
get_latest_version() {
    print_message "$BLUE" "🔍 Ən son versiya yoxlanılır..."
    
    # Try to get latest release from GitHub API
    LATEST_VERSION=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$LATEST_VERSION" ]; then
        print_message "$YELLOW" "⚠️  GitHub API-dən versiya alına bilmədi. v1.0.0 istifadə edilir."
        LATEST_VERSION="v1.0.0"
    fi
    
    print_message "$GREEN" "✅ Versiya: $LATEST_VERSION"
}

# Download binary
download_binary() {
    print_message "$BLUE" "📥 Binary yüklənir..."
    
    # Construct download URL
    BINARY_URL="https://github.com/$GITHUB_REPO/releases/download/$LATEST_VERSION/ubuntu_resource_api-$ARCH-unknown-linux-gnu"
    
    print_message "$YELLOW" "URL: $BINARY_URL"
    
    # Create temp directory
    TMP_DIR=$(mktemp -d)
    cd "$TMP_DIR"
    
    # Download binary
    if ! curl -L -o "$BINARY_NAME" "$BINARY_URL"; then
        print_message "$RED" "❌ Binary yüklənə bilmədi."
        print_message "$YELLOW" "💡 Əgər release yoxdursa, əvvəlcə 'make release' ilə yaradın."
        rm -rf "$TMP_DIR"
        exit 1
    fi
    
    # Make executable
    chmod +x "$BINARY_NAME"
    
    print_message "$GREEN" "✅ Binary yükləndi"
}

# Get user input for configuration
get_configuration() {
    print_message "$BLUE" "⚙️  Konfiqurasiya"
    echo ""
    
    # Get bind address
    read -p "Server adresi (default: $DEFAULT_ADDRESS): " BIND_ADDRESS
    BIND_ADDRESS=${BIND_ADDRESS:-$DEFAULT_ADDRESS}
    
    # Get port
    read -p "Server portu (default: $DEFAULT_PORT): " PORT
    PORT=${PORT:-$DEFAULT_PORT}
    
    # Nginx paths
    read -p "Nginx sites-available yolu (default: /etc/nginx/sites-available): " NGINX_AVAILABLE
    NGINX_AVAILABLE=${NGINX_AVAILABLE:-/etc/nginx/sites-available}
    
    read -p "Nginx sites-enabled yolu (default: /etc/nginx/sites-enabled): " NGINX_ENABLED
    NGINX_ENABLED=${NGINX_ENABLED:-/etc/nginx/sites-enabled}
    
    # Docker socket
    read -p "Docker socket yolu (default: unix:///var/run/docker.sock): " DOCKER_SOCKET
    DOCKER_SOCKET=${DOCKER_SOCKET:-unix:///var/run/docker.sock}
    
    echo ""
    print_message "$GREEN" "📝 Konfiqurasiya:"
    echo "   Server: $BIND_ADDRESS:$PORT"
    echo "   Nginx Available: $NGINX_AVAILABLE"
    echo "   Nginx Enabled: $NGINX_ENABLED"
    echo "   Docker Socket: $DOCKER_SOCKET"
    echo ""
    
    read -p "Davam etmək istəyirsiniz? (y/n): " CONFIRM
    if [[ ! $CONFIRM =~ ^[Yy]$ ]]; then
        print_message "$YELLOW" "Quraşdırma ləğv edildi."
        rm -rf "$TMP_DIR"
        exit 0
    fi
}

# Install the application
install_application() {
    print_message "$BLUE" "📦 Tətbiq quraşdırılır..."
    
    # Create installation directory
    mkdir -p "$INSTALL_DIR"
    
    # Copy binary
    cp "$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Clean up temp directory
    cd /
    rm -rf "$TMP_DIR"
    
    print_message "$GREEN" "✅ Tətbiq quraşdırıldı: $INSTALL_DIR"
}

# Create systemd service
create_service() {
    print_message "$BLUE" "🔧 Systemd servisi yaradılır..."
    
    cat > "/etc/systemd/system/${SERVICE_NAME}.service" <<EOF
[Unit]
Description=Ubuntu Resource Monitor
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/$BINARY_NAME
Restart=always
RestartSec=10

# Environment variables
Environment="SERVER_BIND_ADDRESS=$BIND_ADDRESS:$PORT"
Environment="NGINX_SITES_AVAILABLE=$NGINX_AVAILABLE"
Environment="NGINX_SITES_ENABLED=$NGINX_ENABLED"
Environment="DOCKER_SOCKET_PATH=$DOCKER_SOCKET"
Environment="RUST_LOG=info"

# Security settings
NoNewPrivileges=false
PrivateTmp=false

[Install]
WantedBy=multi-user.target
EOF
    
    # Reload systemd
    systemctl daemon-reload
    
    print_message "$GREEN" "✅ Systemd servisi yaradıldı"
}

# Start the service
start_service() {
    print_message "$BLUE" "🚀 Servis başladılır..."
    
    # Enable service to start on boot
    systemctl enable "$SERVICE_NAME"
    
    # Start service
    systemctl start "$SERVICE_NAME"
    
    # Wait a moment for service to start
    sleep 2
    
    # Check status
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        print_message "$GREEN" "✅ Servis uğurla başladıldı!"
    else
        print_message "$RED" "❌ Servis başlamadı. Status yoxlayın: systemctl status $SERVICE_NAME"
        exit 1
    fi
}

# Print success message
print_success() {
    echo ""
    print_message "$GREEN" "🎉 Quraşdırma tamamlandı!"
    echo ""
    print_message "$BLUE" "📊 Dashboard: http://$BIND_ADDRESS:$PORT/dashboard"
    print_message "$BLUE" "🔄 Nginx Manager: http://$BIND_ADDRESS:$PORT/nginx"
    print_message "$BLUE" "🐳 Docker Manager: http://$BIND_ADDRESS:$PORT/docker"
    echo ""
    print_message "$YELLOW" "Faydalı əmrlər:"
    echo "  Servisi dayandır:       sudo systemctl stop $SERVICE_NAME"
    echo "  Servisi başlat:         sudo systemctl start $SERVICE_NAME"
    echo "  Servisi yenidən başlat: sudo systemctl restart $SERVICE_NAME"
    echo "  Status yoxla:           sudo systemctl status $SERVICE_NAME"
    echo "  Logları gör:            sudo journalctl -u $SERVICE_NAME -f"
    echo "  Servisi sil:            sudo bash uninstall.sh"
    echo ""
}

# Main installation flow
main() {
    print_message "$GREEN" "╔════════════════════════════════════════╗"
    print_message "$GREEN" "║  Ubuntu Resource Monitor - Installer  ║"
    print_message "$GREEN" "║      (Binary Installation - No Rust)  ║"
    print_message "$GREEN" "╚════════════════════════════════════════╝"
    echo ""
    
    check_root
    detect_arch
    get_latest_version
    get_configuration
    download_binary
    install_application
    create_service
    start_service
    print_success
}

# Run main function
main
