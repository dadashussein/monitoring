#!/bin/bash

# Ubuntu Resource Monitor - Installation Script
# This script installs the application as a systemd service

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
DEFAULT_PORT="8080"
DEFAULT_ADDRESS="0.0.0.0"
INSTALL_DIR="/opt/ubuntu-resource-monitor"
SERVICE_NAME="ubuntu-resource-monitor"
BINARY_NAME="ubuntu_resource_api"

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

# Check system requirements
check_requirements() {
    print_message "$BLUE" "🔍 Sistem tələbləri yoxlanılır..."
    
    # Check if systemd is available
    if ! command -v systemctl &> /dev/null; then
        print_message "$RED" "❌ systemd tapılmadı. Bu sistem dəstəklənmir."
        exit 1
    fi
    
    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        print_message "$YELLOW" "⚠️  Rust/Cargo tapılmadı. Quraşdırılır..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    print_message "$GREEN" "✅ Sistem tələbləri ödənildi"
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
        exit 0
    fi
}

# Build the application
build_application() {
    print_message "$BLUE" "🔨 Tətbiq build edilir..."
    
    if [ ! -f "Cargo.toml" ]; then
        print_message "$RED" "❌ Cargo.toml tapılmadı. Layihə qovluğunda olduğunuzdan əmin olun."
        exit 1
    fi
    
    cargo build --release
    
    if [ ! -f "target/release/$BINARY_NAME" ]; then
        print_message "$RED" "❌ Build uğursuz oldu."
        exit 1
    fi
    
    print_message "$GREEN" "✅ Build tamamlandı"
}

# Install the application
install_application() {
    print_message "$BLUE" "📦 Tətbiq quraşdırılır..."
    
    # Create installation directory
    mkdir -p "$INSTALL_DIR"
    
    # Copy binary
    cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Create templates directory
    mkdir -p "$INSTALL_DIR/templates"
    if [ -d "src/templates" ]; then
        cp src/templates/*.html "$INSTALL_DIR/templates/" 2>/dev/null || true
    fi
    
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
    echo "  Servisi dayandır:    sudo systemctl stop $SERVICE_NAME"
    echo "  Servisi başlat:      sudo systemctl start $SERVICE_NAME"
    echo "  Servisi yenidən başlat: sudo systemctl restart $SERVICE_NAME"
    echo "  Status yoxla:        sudo systemctl status $SERVICE_NAME"
    echo "  Logları gör:         sudo journalctl -u $SERVICE_NAME -f"
    echo "  Servisi sil:         sudo systemctl stop $SERVICE_NAME && sudo systemctl disable $SERVICE_NAME"
    echo ""
}

# Main installation flow
main() {
    print_message "$GREEN" "╔════════════════════════════════════════╗"
    print_message "$GREEN" "║  Ubuntu Resource Monitor - Installer  ║"
    print_message "$GREEN" "╚════════════════════════════════════════╝"
    echo ""
    
    check_root
    check_requirements
    get_configuration
    build_application
    install_application
    create_service
    start_service
    print_success
}

# Run main function
main
