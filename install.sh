#!/bin/bash
set -e

REPO_URL="https://github.com/yourusername/ubuntu-resource-api"
BINARY_NAME="ubuntu_resource_api"
INSTALL_DIR="/usr/local/bin"
SERVICE_NAME="ubuntu-resource-api"
VERSION="${VERSION:-latest}"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║          Ubuntu Resource API - Installer                      ║"
    echo "║          System Monitoring Dashboard                          ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

check_root() {
    if [ "$EUID" -ne 0 ] && [ "$USER_INSTALL" != "true" ]; then
        log_error "Please run as root or with sudo:"
        echo "  curl -sSL https://your-domain.com/install.sh | sudo bash"
        exit 1
    fi
}

detect_arch() {
    ARCH=$(uname -m)
    case $ARCH in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="aarch64" ;;
        *) log_error "Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    log_info "Architecture: $ARCH"
}

download_binary() {
    log_info "Downloading $BINARY_NAME..."
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT
    
    # Change this URL to your actual hosting
    DOWNLOAD_URL="https://your-domain.com/download/${BINARY_NAME}-linux-${ARCH}"
    
    if ! curl -fsSL -o "$TMP_DIR/$BINARY_NAME" "$DOWNLOAD_URL" 2>/dev/null; then
        log_warn "Binary download failed, building from source..."
        build_from_source "$TMP_DIR"
    fi
    
    chmod +x "$TMP_DIR/$BINARY_NAME"
    BINARY_PATH="$TMP_DIR/$BINARY_NAME"
}

build_from_source() {
    local DEST_DIR="$1"
    log_info "Installing Rust (if needed)..."
    if ! command -v cargo &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi
    log_info "Building from source (may take a few minutes)..."
    TMP_BUILD=$(mktemp -d)
    cd "$TMP_BUILD"
    git clone --depth 1 "$REPO_URL" 2>/dev/null || { log_error "Failed to clone repo"; exit 1; }
    cd ubuntu-resource-api
    cargo build --release 2>&1 | tail -5
    cp target/release/$BINARY_NAME "$DEST_DIR/"
    rm -rf "$TMP_BUILD"
}

install_binary() {
    log_info "Installing to $INSTALL_DIR..."
    if [ "$USER_INSTALL" = "true" ]; then
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
        if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
            echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$HOME/.bashrc"
            log_warn "Added $INSTALL_DIR to PATH. Run 'source ~/.bashrc' to apply."
        fi
    else
        cp "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
    fi
    log_success "Binary installed"
}

install_service() {
    [ "$USER_INSTALL" = "true" ] && return
    log_info "Creating systemd service..."
    cat > /etc/systemd/system/${SERVICE_NAME}.service << 'EOF'
[Unit]
Description=Ubuntu Resource API
After=network.target
[Service]
Type=simple
ExecStart=/usr/local/bin/ubuntu_resource_api
Restart=always
RestartSec=5
User=nobody
[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload
    systemctl enable --now $SERVICE_NAME
    log_success "Service started"
}

configure_firewall() {
    [ "$USER_INSTALL" = "true" ] && return
    if command -v ufw &>/dev/null && ufw status | grep -q "active"; then
        ufw allow 8080/tcp >/dev/null 2>&1 && log_success "Firewall configured (UFW)"
    elif command -v firewall-cmd &>/dev/null; then
        firewall-cmd --permanent --add-port=8080/tcp >/dev/null 2>&1
        firewall-cmd --reload >/dev/null 2>&1 && log_success "Firewall configured (firewalld)"
    fi
}

print_success() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                    Installation Complete!                     ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    log_info "Dashboard: http://$(hostname -I | awk '{print $1}'):8080/dashboard"
    log_info "API:       http://$(hostname -I | awk '{print $1}'):8080/"
    echo ""
    [ "$USER_INSTALL" != "true" ] && log_info "Service status: systemctl status $SERVICE_NAME"
    log_info "To uninstall: sudo $INSTALL_DIR/$BINARY_NAME --uninstall"
}

main() {
    print_banner
    detect_arch
    [ "$SKIP_ROOT_CHECK" != "true" ] && check_root
    download_binary
    install_binary
    install_service
    configure_firewall
    print_success
}

main "$@"
