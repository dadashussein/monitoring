#!/bin/bash

# Ubuntu Resource Monitor - Uninstallation Script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

INSTALL_DIR="/opt/ubuntu-resource-monitor"
SERVICE_NAME="ubuntu-resource-monitor"

print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    print_message "$RED" "❌ Bu skript root olaraq çalışdırılmalıdır. 'sudo' istifadə edin."
    exit 1
fi

print_message "$BLUE" "🗑️  Ubuntu Resource Monitor silinir..."
echo ""

# Stop service
if systemctl is-active --quiet "$SERVICE_NAME"; then
    print_message "$YELLOW" "⏹️  Servis dayandırılır..."
    systemctl stop "$SERVICE_NAME"
fi

# Disable service
if systemctl is-enabled --quiet "$SERVICE_NAME" 2>/dev/null; then
    print_message "$YELLOW" "🔓 Servis deaktiv edilir..."
    systemctl disable "$SERVICE_NAME"
fi

# Remove service file
if [ -f "/etc/systemd/system/${SERVICE_NAME}.service" ]; then
    print_message "$YELLOW" "📝 Servis faylı silinir..."
    rm "/etc/systemd/system/${SERVICE_NAME}.service"
    systemctl daemon-reload
fi

# Remove installation directory
if [ -d "$INSTALL_DIR" ]; then
    print_message "$YELLOW" "📦 Quraşdırma qovluğu silinir..."
    rm -rf "$INSTALL_DIR"
fi

print_message "$GREEN" "✅ Ubuntu Resource Monitor uğurla silindi!"
echo ""
